pub struct Config {
    pub cell_size: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HeatMap {
    #[serde(default)]
    min_x: usize,
    #[serde(default)]
    min_y: usize,
    max_x: usize,
    max_y: usize,
    max_value: usize,
    rows: Vec<Vec<usize>>,
    block_size: f32,
}

impl HeatMap {
    fn new(block_size: f32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
            max_value: 0,
            rows: Vec::new(),
            block_size,
        }
    }

    fn increment(&mut self, x: usize, y: usize) {
        if self.rows.len() <= y  {
                self.rows.resize(y + 1, Vec::new());
            }

            self.max_y = self.max_y.max(y);
            let row = self.rows.get_mut(y ).unwrap();

        if row.len() <= x {
                row.resize(x + 1, 0);
            }

            self.max_x = self.max_x.max(x);
            let cell = row.get_mut(x).unwrap();


        *cell += 1;

        self.max_value = self.max_value.max(*cell);
    }
}

#[derive(Debug)]
pub struct HeatMapOutput {
    pub player_heatmaps: std::collections::HashMap<(csdemo::UserId, String), HeatMap>,
    pub player_info: std::collections::HashMap<csdemo::UserId, csdemo::parser::Player>,
}

#[derive(Debug)]
pub struct Team {
    pub num: u32,
    pub name: String,
    pub players: Vec<u32>,
    pub pawns: Vec<PawnID>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PawnID(u32);

impl From<i32> for PawnID {
    fn from(value: i32) -> Self {
        Self((value & 0x7FF) as u32)
    }
}
impl From<u32> for PawnID {
    fn from(value: u32) -> Self {
        Self(value & 0x7FF)
    }
}

pub fn parse(config: &Config, buf: &[u8]) -> Result<HeatMapOutput, ()> {
    let tmp = csdemo::Container::parse(buf).map_err(|e| ())?;
    let output = csdemo::parser::parse(
        csdemo::FrameIterator::parse(tmp.inner),
        csdemo::parser::EntityFilter::all(),
    )
    .map_err(|e| ())?;

    let pawn_ids = {
        let mut tmp = std::collections::HashMap::<PawnID,_>::new();
        
        for event in output.events.iter() {
            let entry = match event {
                csdemo::DemoEvent::GameEvent(ge) => match ge.as_ref() {
                    csdemo::game_event::GameEvent::PlayerSpawn(pspawn) => match pspawn.userid_pawn.as_ref() {
                        Some(csdemo::RawValue::I32(v)) => {
                            Some((PawnID::from(*v), pspawn.userid.unwrap()))
                        }
                        _ => {
                            None
                        },
                    },
                    _ => None,
                },
                _ => None,
            };

            if let Some((pawn, userid)) = entry {
                if let Some(previous) = tmp.insert(pawn, userid){
                    assert_eq!(previous, userid);
                }
            }
        }

        tmp
    };

    let mut teams = std::collections::HashMap::new();
    let mut player_lifestate = std::collections::HashMap::<csdemo::UserId, u32>::new();
    let mut player_position = std::collections::HashMap::<csdemo::UserId, (f32, f32, f32)>::new();
    let mut player_cells = std::collections::HashMap::new();

    let mut heatmaps = std::collections::HashMap::new();
    for tick_state in output.entity_states.ticks.iter() {
        let _tracing_guard = tracing::debug_span!("Tick", tick=?tick_state.tick).entered();

        process_tick(
            config,
            tick_state,
            &pawn_ids,
            &mut teams,
            &mut player_lifestate,
            &mut player_position,
            &mut player_cells,
            &mut heatmaps,
        );
    }

    tracing::debug!("Pawn-IDs: {:?}", pawn_ids);

    Ok(HeatMapOutput {
        player_heatmaps: heatmaps,
        player_info: output.player_info,
    })
}

pub const MAX_COORD: f32 = (1 << 14) as f32;

fn process_tick(
    config: &Config,
    tick_state: &csdemo::parser::EntityTickStates,
    pawn_ids: &std::collections::HashMap<PawnID, csdemo::UserId>,
    teams: &mut std::collections::HashMap<PawnID, String>,
    player_lifestate: &mut std::collections::HashMap<csdemo::UserId, u32>,
    player_position: &mut std::collections::HashMap<csdemo::UserId, (f32, f32, f32)>,
    player_cells: &mut std::collections::HashMap<csdemo::UserId, (u32, u32, u32)>,
    heatmaps: &mut std::collections::HashMap<(csdemo::UserId, String), HeatMap>,
) {
    for entity_state in tick_state
        .states
        .iter()
        .filter(|s| matches!(s.class.as_ref(), "CCSPlayerPawn" | "CCSTeam"))
    {
        if entity_state.class.as_ref() == "CCSTeam" {
            let raw_team_name = match entity_state.get_prop("CCSTeam.m_szTeamname").map(|p| match &p.value {
                csdemo::parser::Variant::String(v) => Some(v),
                _ => None,
            }).flatten() {
                Some(n) => n,
                None => continue,
            };

            for prop in entity_state.props.iter().filter(|p| p.prop_info.prop_name.as_ref() == "CCSTeam.m_aPawns").filter_map(|p| p.value.as_u32().map(|v| PawnID::from(v))) {
                teams.insert(prop, raw_team_name.clone());
            }

            continue;
        }

        let pawn_id = PawnID::from(entity_state.id);
        let user_id = match pawn_ids.get(&pawn_id).cloned() {
            Some(id) => id,
            None => continue,
        };
        let team = match teams.get(&pawn_id).cloned() {
            Some(t) => t,
            None => continue,
        };

        let _inner_guard =
            tracing::trace_span!("Entity", entity_id=?entity_state.id).entered();

        let x_cell = match entity_state.get_prop("CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellX").map(|prop| prop.value.as_u32()).flatten() {
            Some(c) => c,
            None => player_cells.get(&user_id).map(|(x, _, _)| *x).unwrap_or(0),
        };
        let y_cell = match entity_state.get_prop("CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellY").map(|prop| prop.value.as_u32()).flatten() {
            Some(c) => c,
            None => player_cells.get(&user_id).map(|(_, y, _)| *y).unwrap_or(0),
        };
        let z_cell = match entity_state.get_prop("CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_cellZ").map(|prop| prop.value.as_u32()).flatten() {
            Some(c) => c,
            None => player_cells.get(&user_id).map(|(_, _, z)| *z).unwrap_or(0),
        };

        player_cells.insert(user_id, (x_cell, y_cell, z_cell));

        let x_coord = match entity_state.get_prop("CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecX").map(|prop| prop.value.as_f32()).flatten() {
            Some(c) => c,
            None => player_position.get(&user_id).map(|(x, _, _)| *x).unwrap_or(0.0),
        };
        let y_coord = match entity_state.get_prop("CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecY").map(|prop| prop.value.as_f32()).flatten() {
            Some(c) => c,
            None => player_position.get(&user_id).map(|(_, y, _)| *y).unwrap_or(0.0),
        };
        let z_coord = match entity_state.get_prop("CCSPlayerPawn.CBodyComponentBaseAnimGraph.m_vecZ").map(|prop| prop.value.as_f32()).flatten() {
            Some(c) => c,
            None => player_position.get(&user_id).map(|(_, _, z)| *z).unwrap_or(0.0),
        };

        player_position.insert(user_id, (x_coord, y_coord, z_coord));

        assert!(x_coord >= 0.0);
        assert!(y_coord >= 0.0);
        assert!(z_coord >= 0.0);

        let x_cell_coord = ((x_cell as f32 * (1 << 9) as f32)) as f32;
        let y_cell_coord = ((y_cell as f32 * (1 << 9) as f32)) as f32;
        let z_cell_coord = ((z_cell as f32 * (1 << 9) as f32)) as f32;

        let x_coord = x_cell_coord + x_coord;
        let y_coord = y_cell_coord + y_coord;
        let z_coord = z_cell_coord + z_coord;

        assert!(x_coord >= 0.0);
        assert!(y_coord >= 0.0);
        assert!(z_coord >= 0.0);

        let x_cell = (x_coord  / config.cell_size) as usize;
        let y_cell = (y_coord / config.cell_size) as usize;

        let n_lifestate = entity_state.props.iter().find_map(|prop| {
            if prop.prop_info.prop_name.as_ref() != "CCSPlayerPawn.m_lifeState" {
                return None;
            }

            match prop.value {
                csdemo::parser::Variant::U32(v) => Some(v),
                _ => None,
            }
        });

        let lifestate = match n_lifestate {
            Some(state) => {
                player_lifestate.insert(user_id, state);
                state
            }
            None => player_lifestate.get(&user_id).copied().unwrap_or(1),
        };

        // 0 means alive
        if lifestate != 0 {
            continue;
        }

        // tracing::trace!("Coord (X, Y, Z): {:?} -> {:?}", (x_coord, y_coord, z_coord), (x_cell, y_cell));

        let heatmap = heatmaps.entry((user_id.clone(), team)).or_insert(HeatMap::new(config.cell_size));
        heatmap.increment(x_cell, y_cell);
    }
}

impl core::fmt::Display for HeatMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.max_value.ilog10() as usize + 1;

        for row in self.rows.iter() {
            for cell in row.iter().copied() {
                write!(f, "{: ^width$} ", cell, width=size)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl HeatMap {
    pub fn coords(&self) -> ((f32, f32), (f32, f32)) {
        (
            (self.min_x as f32 * self.block_size - MAX_COORD, self.max_x as f32 * self.block_size - MAX_COORD),
            (self.min_y as f32 * self.block_size - MAX_COORD, self.max_y as f32 * self.block_size - MAX_COORD)
        )
    }

    pub fn as_image(&self) -> image::RgbImage {
        use colors_transform::Color;

        let mut buffer = image::RgbImage::new((self.max_x - self.min_x) as u32 + 1, (self.max_y - self.min_y) as u32 + 1);

        for (y, row) in self.rows.iter().rev().enumerate() {
            for (x, cell) in row.iter().copied().chain(core::iter::repeat(0)).enumerate().take(self.max_x - self.min_x) {
                let scaled = (1.0/(1.0 + (cell as f32))) * 240.0;
                let raw_rgb = colors_transform::Hsl::from(scaled, 100.0, 50.0).to_rgb();

                buffer.put_pixel(x as u32, y as u32, image::Rgb([raw_rgb.get_red() as u8, raw_rgb.get_green() as u8, raw_rgb.get_blue() as u8]))
            }
        }

        buffer
    }

    pub fn fit(&mut self, xs: core::ops::Range<f32>, ys: core::ops::Range<f32>) {
        let min_x = (xs.start / self.block_size - self.min_x as f32) as usize;       
        let min_y = (ys.start / self.block_size - self.min_y as f32) as usize;

        let _ = self.rows.drain(0..min_y);
        for row in self.rows.iter_mut() {
            let _ = row.drain(0..min_x.min(row.len()));
        }

        let x_steps = ((xs.end - xs.start) / self.block_size) as usize;
        let y_steps = ((ys.end - ys.start) / self.block_size) as usize;

        for row in self.rows.iter_mut() {
            row.resize(x_steps, 0);
        }
        self.rows.resize_with(y_steps, || vec![0; x_steps]);

        self.min_y += (0..min_y).len();
        self.min_x += (0..min_x).len();

        self.max_y = self.min_y + self.rows.len();
        self.max_x = self.min_x + self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_no_cutoff() {
        let mut input = HeatMap::new(2.0);

        input.increment(3, 3);
        input.increment(2, 2);

        assert_eq!(input.min_x, 0);
        assert_eq!(input.min_y, 0);
        assert_eq!(input.max_x, 3);
        assert_eq!(input.max_y, 3);

        assert_eq!(
            &vec![
                vec![],
                vec![],
                vec![0, 0, 1],
                vec![0, 0, 0, 1]
            ],
            &input.rows
        );

        input.fit(2.0..10.0, 2.0..10.0);

        
        assert_eq!(
            &vec![
                vec![0, 0, 0, 0],
                vec![0, 1, 0, 0],
                vec![0, 0, 1, 0],
                vec![0, 0, 0, 0],
            ],
            &input.rows
        );
    }

    #[test]
    fn fit_cutoff() {
        let mut input = HeatMap::new(2.0);

        input.increment(3, 3);
        input.increment(2, 2);

        assert_eq!(input.min_x, 0);
        assert_eq!(input.min_y, 0);
        assert_eq!(input.max_x, 3);
        assert_eq!(input.max_y, 3);

        assert_eq!(
            &vec![
                vec![],
                vec![],
                vec![0, 0, 1],
                vec![0, 0, 0, 1]
            ],
            &input.rows
        );

        input.fit(6.0..10.0, 6.0..10.0);

        assert_eq!(
            &vec![
                vec![1, 0],
                vec![0, 0]
            ],
            &input.rows
        );
    }
}
