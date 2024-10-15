#[derive(Debug, PartialEq)]
pub struct EndOfGame {
    pub map: String,
    pub players: Vec<(PlayerInfo, PlayerStats)>,
    pub teams: std::collections::HashMap<i32, TeamInfo>,
}

#[derive(Debug, PartialEq)]
pub struct TeamInfo {
    pub end_score: usize,
    pub start_side: String,
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
    pub assists: usize,
    pub team_kills: usize,
    pub team_damage: usize,
    pub self_damage: usize,
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
    let mut pawn_to_player = std::collections::HashMap::<csdemo::structured::pawnid::PawnID, csdemo::UserId>::new();

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
                        let userid = pspawn.userid.unwrap();

                        player_life.insert(userid.clone(), 100);

                        if let Some(pawn) = pspawn.userid_pawn.as_ref().map(|p| match p { csdemo::RawValue::I32(v) => Some(csdemo::structured::pawnid::PawnID::from(*v)), _ => None }).flatten() {
                            pawn_to_player.insert(pawn, userid);
                        }
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
                    _ => {}
                };
            }
            _ => {}
        };
    }

    let mut teams = std::collections::HashMap::<i32, TeamInfo>::new();

    let mut entity_to_team = std::collections::HashMap::new();
    for tick_state in output.entity_states.ticks {
        for state in tick_state.states {
            let team = match csdemo::structured::ccsteam::CCSTeam::try_from(&state) {
                Ok(t) => t,
                Err(_) => continue,
            };

            let pawns = team.player_pawns();
            let player_ids = pawns.into_iter().filter_map(|pawn| pawn_to_player.get(&pawn)).collect::<Vec<_>>();
            if player_ids.is_empty() {
                if let Some(team_number) = entity_to_team.get(&team.entity_id()) {
                    if let Some(score) = team.score() {
                        if let Some(team_entry) = teams.get_mut(team_number) {
                            team_entry.end_score = score as usize;
                        }
                    }
                }

                continue;
            }

            let team_number = player_ids.iter().filter_map(|p| output.player_info.get(*p).map(|p| p.team)).next().unwrap();

            entity_to_team.insert(team.entity_id(), team_number);
            
            let team_entry = teams.entry(team_number).or_insert_with(|| {
                TeamInfo {
                    end_score: 0,
                    start_side: team.team_name().map(|t| t.to_owned()).unwrap_or(String::new()),
                }
            });
            if let Some(score) = team.score() {
                team_entry.end_score = score as usize;
            }
        }
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

    Ok(EndOfGame {
        map,
        players,
        teams,
    })
}

fn player_death(
    death: &csdemo::game_event::PlayerDeath,
    player_info: &std::collections::HashMap<csdemo::UserId, csdemo::parser::Player>,
    player_stats: &mut std::collections::HashMap<csdemo::UserId, PlayerStats>,
) {
    let player_died_id = death.userid.unwrap();

    let player_died_player = player_info.get(&player_died_id).unwrap();
    let player_died = player_stats.entry(player_died_id).or_default();

    let attacker_id = match death.attacker.filter(|p| p.0 < 10) {
        Some(a) => a,
        None => {
            return;
        }
    };

    player_died.deaths += 1;

    let attacker_player = player_info
        .get(&attacker_id)
        .expect(&format!("Attacker-ID: {:?}", attacker_id));
    if attacker_player.xuid == player_died_player.xuid {
        // TODO
        // Player committed Suicide
        // How to handle?
    } else if attacker_player.team == player_died_player.team {
        let attacker = player_stats.entry(attacker_id).or_default();
        attacker.team_kills += 1;
    } else {
        let attacker = player_stats.entry(attacker_id).or_default();
        attacker.kills += 1;
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
        None => {
            return;
        }
    };

    let attacker_id = match hurt.attacker {
        Some(aid) => aid,
        None => {
            return;
        }
    };

    let n_health = match hurt.health {
        Some(csdemo::RawValue::F32(v)) => v as u8,
        Some(csdemo::RawValue::I32(v)) => v as u8,
        Some(csdemo::RawValue::U64(v)) => v as u8,
        _ => 0,
    };
    let previous_health = player_life
        .get(hurt.userid.as_ref().unwrap())
        .copied()
        .unwrap();
    let dmg_dealt = previous_health - n_health;

    player_life.insert(hurt.userid.unwrap(), n_health);

    if let Some(attacking_player) = player_info.get(&attacker_id) {
        let attacker = player_stats.entry(attacker_id).or_default();

        if attacking_player.xuid == attacked_player.xuid {
            attacker.self_damage += dmg_dealt as usize;
        } else if attacking_player.team == attacked_player.team {
            attacker.team_damage += dmg_dealt as usize;
        } else {
            attacker.damage += dmg_dealt as usize;
        }
    }
}
