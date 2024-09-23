#[derive(Debug, PartialEq)]
pub struct EndOfGame {
    pub map: String,
    pub players: Vec<(PlayerInfo, PlayerStats)>,
}

#[derive(Debug, PartialEq)]
pub struct PlayerInfo {
    pub name: String,
    pub steam_id: String,
    pub team: i32,
    pub color: i32,
    pub ingame_id: i32,
}

#[derive(Debug, Default, PartialEq)]
pub struct PlayerStats {
    pub kills: usize,
    pub deaths: usize,
    pub damage: usize,
    pub team_damage: usize,
    pub self_damage: usize,
    pub assists: usize,
}

pub fn parse(buf: &[u8]) -> Result<EndOfGame, ()> {
    let tmp = csdemo::Container::parse(buf).map_err(|e| ())?;
    let output = csdemo::parser::parse(
        csdemo::FrameIterator::parse(tmp.inner),
        csdemo::parser::EntityFilter::all(),
    )
    .map_err(|e| ())?;

    let header = &output.header;

    let mut player_stats = std::collections::HashMap::<_, PlayerStats>::new();

    let mut track = false;
    let mut player_life = std::collections::HashMap::<_, u8>::new();
    for event in output.events.iter() {
        match event {
            csdemo::DemoEvent::GameEvent(gevent) => {
                match gevent.as_ref() {
                    csdemo::game_event::GameEvent::RoundAnnounceMatchStart(_) => {
                        player_stats.clear();
                        track = true;
                    }
                    csdemo::game_event::GameEvent::RoundPreStart(_) => {
                        track = true;
                    }
                    csdemo::game_event::GameEvent::PlayerSpawn(pspawn) => {
                        player_life.insert(pspawn.userid.unwrap(), 100);
                    }
                    csdemo::game_event::GameEvent::WinPanelMatch(_) => {
                        track = false;
                    }
                    csdemo::game_event::GameEvent::RoundOfficiallyEnded(_) => {
                        track = false;
                    }
                    csdemo::game_event::GameEvent::PlayerDeath(pdeath) if track => {
                        player_death(pdeath, &output.player_info, &mut player_stats);
                    }
                    csdemo::game_event::GameEvent::PlayerHurt(phurt) if track => {
                        player_hurt(
                            phurt,
                            &output.player_info,
                            &mut player_stats,
                            &mut player_life,
                        );
                    }
                    other => {}
                };
            }
            _ => {}
        };
    }

    let mut players: Vec<_> = player_stats
        .into_iter()
        .filter_map(|(id, stats)| {
            let player = output.player_info.get(&id)?;

            Some((
                PlayerInfo {
                    name: player.name.clone(),
                    steam_id: player.xuid.to_string(),
                    team: player.team,
                    color: player.color,
                    ingame_id: id.0,
                },
                stats,
            ))
        })
        .collect();
    players.sort_unstable_by_key(|(p, _)| p.ingame_id);

    let map = header.map_name().to_owned();

    Ok(EndOfGame { map, players })
}

fn player_death(
    death: &csdemo::game_event::PlayerDeath,
    player_info: &std::collections::HashMap<csdemo::UserId, csdemo::parser::Player>,
    player_stats: &mut std::collections::HashMap<csdemo::UserId, PlayerStats>,
) {
    let player_died_id = death.userid.unwrap();

    let player_died_player = player_info.get(&player_died_id).unwrap();
    let player_died = player_stats.entry(player_died_id).or_default();
    player_died.deaths += 1;

    if let Some(attacker_id) = death.attacker.filter(|p| p.0 < 10) {
        let attacker_player = player_info
            .get(&attacker_id)
            .expect(&format!("Attacker-ID: {:?}", attacker_id));
        if attacker_player.team == player_died_player.team {
            return;
        } else {
            let attacker = player_stats.entry(attacker_id).or_default();
            attacker.kills += 1;
        }
    }
    if let Some(assist_id) = death.assister.filter(|p| p.0 < 10) {
        let assister_player = player_info
            .get(&assist_id)
            .expect(&format!("Assister-ID: {:?}", assist_id));

        if assister_player.team == player_died_player.team {
        } else {
            let assister = player_stats.entry(assist_id).or_default();
            assister.assists += 1;
        }
    }
}

fn player_hurt(
    hurt: &csdemo::game_event::PlayerHurt,
    player_info: &std::collections::HashMap<csdemo::UserId, csdemo::parser::Player>,
    player_stats: &mut std::collections::HashMap<csdemo::UserId, PlayerStats>,
    player_life: &mut std::collections::HashMap<csdemo::UserId, u8>,
) {
    let attacked_player = match player_info.get(hurt.userid.as_ref().unwrap()) {
        Some(a) => a,
        None => return,
    };

    let attacker_id = match hurt.attacker {
        Some(aid) => aid,
        None => return,
    };

    let attacking_player = match player_info.get(&attacker_id) {
        Some(a) => a,
        None => return,
    };

    let attacker = player_stats.entry(attacker_id).or_default();

    let n_health = match hurt.health {
        Some(csdemo::RawValue::F32(v)) => v as u8,
        Some(csdemo::RawValue::I32(v)) => v as u8,
        Some(csdemo::RawValue::U64(v)) => v as u8,
        _ => 0,
    };
    let dmg_dealt = player_life
        .get(hurt.userid.as_ref().unwrap())
        .copied()
        .unwrap_or(100)
        - n_health;

    player_life.insert(hurt.userid.unwrap(), n_health);

    if attacking_player.team == attacked_player.team {
        return;
    }

    attacker.damage += dmg_dealt as usize;
}
