pub struct Config {
    pub cell_size: f32,
}

pub struct HeatMap {
    max_x: usize,
    max_y: usize,
    max_value: usize,
    rows: Vec<Vec<usize>>,
}

impl HeatMap {
    fn new() -> Self {
        Self {
            max_x: 0,
            max_y: 0,
            max_value: 0,
            rows: Vec::new(),
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

pub fn parse(config: &Config, buf: &[u8]) -> Result<std::collections::HashMap<csdemo::UserId, HeatMap>, ()> {
    let tmp = csdemo::Container::parse(buf).map_err(|e| ())?;
    let output = csdemo::parser::parse(
        csdemo::FrameIterator::parse(tmp.inner),
        csdemo::parser::EntityFilter::all(),
    )
    .map_err(|e| ())?;

    let pawn_ids: std::collections::HashMap<_, _> = output
        .events
        .iter()
        .filter_map(|event| match event {
            csdemo::DemoEvent::GameEvent(ge) => match ge.as_ref() {
                csdemo::game_event::GameEvent::PlayerSpawn(pspawn) => match pspawn.userid_pawn {
                    Some(csdemo::RawValue::I32(v)) => Some((v, pspawn.userid.unwrap())),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .collect();

    let mut entity_id_to_user = std::collections::HashMap::<i32, csdemo::UserId>::new();
    let mut player_lifestate = std::collections::HashMap::<csdemo::UserId, u32>::new();
    let mut player_position = std::collections::HashMap::<csdemo::UserId, (f32, f32, f32)>::new();
    let mut player_cells = std::collections::HashMap::new();

    let mut heatmaps = std::collections::HashMap::new();
    for tick_state in output.entity_states.ticks.iter() {
        let _tracing_guard = tracing::debug_span!("Tick", tick=?tick_state.tick).entered();

        process_tick(
            config,
            tick_state,
            &mut entity_id_to_user,
            &pawn_ids,
            &mut player_lifestate,
            &mut player_position,
            &mut player_cells,
            &mut heatmaps
        );
    }

    Ok(heatmaps)
}

fn process_tick(
    config: &Config,
    tick_state: &csdemo::parser::EntityTickStates,
    entity_id_to_user: &mut std::collections::HashMap<i32, csdemo::UserId>,
    pawn_ids: &std::collections::HashMap<i32, csdemo::UserId>,
    player_lifestate: &mut std::collections::HashMap<csdemo::UserId, u32>,
    player_position: &mut std::collections::HashMap<csdemo::UserId, (f32, f32, f32)>,
    player_cells: &mut std::collections::HashMap<csdemo::UserId, (u32, u32, u32)>,
    heatmaps: &mut std::collections::HashMap<csdemo::UserId, HeatMap>,
) {
    for entity_state in tick_state
        .states
        .iter()
        .filter(|s| s.class == "CCSPlayerPawn")
    {
        let user_id_entry = entity_id_to_user.entry(entity_state.id);
        let user_id = match user_id_entry {
            std::collections::hash_map::Entry::Occupied(v) => v.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => {
                let pawn_id_prop: Option<i32> = entity_state.props.iter().find_map(|prop| {
                    if prop.prop_info.prop_name.as_ref() != "CCSPlayerPawn.m_nEntityId" {
                        return None;
                    }

                    let pawn_id: i32 = match &prop.value {
                        csdemo::parser::Variant::U32(v) => *v as i32,
                        other => panic!("Unexpected Variant: {:?}", other),
                    };

                    Some(pawn_id)
                });

                let user_id: Option<csdemo::UserId> = pawn_id_prop
                    .map(|pawn_id| pawn_ids.get(&pawn_id).cloned())
                    .flatten();

                match user_id {
                    Some(user_id) => v.insert(user_id),
                    None => continue,
                }
            }
        };

        let _inner_guard =
            tracing::trace_span!("Entity", ?user_id, entity_id=?entity_state.id).entered();

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

        player_cells.insert(*user_id, (x_cell, y_cell, z_cell));

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

        player_position.insert(*user_id, (x_coord, y_coord, z_coord));

        assert!(x_coord >= 0.0);
        assert!(y_coord >= 0.0);
        assert!(z_coord >= 0.0);

        const MAX_COORD: f32 = (1 << 14) as f32;

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
                player_lifestate.insert(*user_id, state);
                state
            }
            None => player_lifestate.get(&user_id).copied().unwrap_or(1),
        };

        // 0 means alive
        if lifestate != 0 {
            continue;
        }

        tracing::trace!("Coord (X, Y, Z): {:?} -> {:?}", (x_coord, y_coord, z_coord), (x_cell, y_cell));

        let heatmap = heatmaps.entry(user_id.clone()).or_insert(HeatMap::new());
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
    pub fn as_image(&self) -> image::RgbImage {
        let mut buffer = image::RgbImage::new(self.max_x as u32 + 1, self.max_y as u32 + 1);

        tracing::trace!("Creating Image with Dimensions: {}x{}", buffer.width(), buffer.height());

        for (y, row) in self.rows.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                buffer.put_pixel(x as u32, y as u32, image::Rgb([*cell as u8, 0, 0]))
            }
        }

        buffer
    }

    pub fn shrink(&mut self) {
        let min_x = self.rows.iter().filter_map(|row| row.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, _)| i).next()).min().unwrap_or(0);
        let min_y = self.rows.iter().enumerate().filter(|(y, row)| row.iter().any(|v| *v != 0)).map(|(i, _)| i).min().unwrap_or(0);

        tracing::trace!("Truncate to Min-X: {} - Min-Y: {}", min_x, min_y);

        let _ = self.rows.drain(0..min_y);
        for row in self.rows.iter_mut() {
            let _ = row.drain(0..min_x);
        }


        self.max_y = self.rows.len();
        self.max_x = self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    }
}
