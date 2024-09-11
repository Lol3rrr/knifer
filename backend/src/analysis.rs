use std::path::PathBuf;

#[derive(Debug)]
pub struct AnalysisInput {
    pub steamid: String,
    pub demoid: i64,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct BaseInfo {
    pub map: String,
}

#[tracing::instrument(skip(input))]
pub fn analyse_base(input: AnalysisInput) -> BaseInfo {
    tracing::info!("Performing Base analysis");

    let huf = l_demoparser::second_pass::parser_settings::create_huffman_lookup_table();
    let settings = l_demoparser::first_pass::parser_settings::ParserInputs {
        wanted_players: Vec::new(),
        real_name_to_og_name: ahash::AHashMap::default(),
        wanted_player_props: vec!["X".to_string(), "team_num".to_string()],
        wanted_events: vec!["player_death".to_string(), "player_team".to_string(), "team_info".to_string(), "player_spawn".to_string(), "team_score".to_string(), "round_end".to_string(), "game_end".to_string(), "match_end_conditions".to_string(), "switch_team".to_string(), "player_given_c4".to_string()],
        wanted_other_props: vec![],
        parse_ents: true,
        wanted_ticks: Vec::new(),
        parse_projectiles: false,
        only_header: false,
        count_props: false,
        only_convars: false,
        huffman_lookup_table: &huf,
        order_by_steamid: false,
    };

    let mut ds = l_demoparser::parse_demo::Parser::new(settings, l_demoparser::parse_demo::ParsingMode::ForceSingleThreaded);
    let file = std::fs::File::open(&input.path).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };
    let output = ds.parse_demo(&mmap).unwrap();

    let header = output.header.as_ref().unwrap();

    tracing::info!("Header: {:?}", header);

    for event in output.game_events.iter() {
        match event.name.as_str() {
            "player_team" => {
                let team = event.fields.iter().find_map(|f| match (f.name.as_str(), &f.data) {
                    ("team", Some(d)) => Some(d),
                    ("team", None) => {
                        tracing::warn!("'team' field without data");
                        None
                    }
                    _ => None,
                });
                let user_name = event.fields.iter().find_map(|f| match (f.name.as_str(), &f.data) {
                    ("user_name", Some(d)) => Some(d),
                    ("user_name", None) => {
                        tracing::warn!("'user_name' field without data");
                        None
                    }
                    _ => None,
                });
                let steamid = event.fields.iter().find_map(|f| match (f.name.as_str(), &f.data) {
                    ("user_steamid", Some(d)) => Some(d),
                    ("user_steamid", None) => {
                        tracing::warn!("'user_steamid' field without data");
                        None
                    }
                    _ => None,
                });

                tracing::info!("'{:?}' ({:?}) -> {:?}", user_name, steamid, team);
            }
            "team_info" => {
                tracing::info!("Team Info: {:?}", event);
            }
            "player_spawn" => {
                // tracing::info!("Player Spawn: {:?}", event);
            }
            "team_score" => {
                tracing::info!("Team Score: {:?}", event);
            }
            "round_end" => {
                let winner = event.fields.iter().find_map(|f| match (f.name.as_str(), &f.data) {
                    ("winner", Some(d)) => Some(d),
                    ("winner", None) => {
                        tracing::warn!("'winner' field without data");
                        None
                    }
                    _ => None,
                });
                let round = event.fields.iter().find_map(|f| match (f.name.as_str(), &f.data) {
                    ("round", Some(d)) => Some(d),
                    ("round", None) => {
                        tracing::warn!("'round' field without data");
                        None
                    }
                    _ => None,
                });
                let reason = event.fields.iter().find_map(|f| match (f.name.as_str(), &f.data) {
                    ("reason", Some(d)) => Some(d),
                    ("reason", None) => {
                        tracing::warn!("'reason' field without data");
                        None
                    }
                    _ => None,
                });

                tracing::info!(?winner, ?round, ?reason, "Round End");
            }
            "game_end" => {
                tracing::info!("Game End: {:?}", event);
            }
            "match_end_conditions" => {
                tracing::info!("Match End Conditions: {:?}", event);
            }
            "switch_team" => {
                tracing::info!("Switch Team: {:?}", event);
            }
            "player_given_c4" => {
                tracing::info!("Player Given C4: {:?}", event);
            }
            _ => {}
        };
    }

    BaseInfo {
        map: header.get("map_name").cloned().unwrap_or_default()
    }
}
