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
    pub players: Vec<()>,
}

#[tracing::instrument(skip(input))]
pub fn analyse_base(input: AnalysisInput) -> BaseInfo {
    tracing::info!("Performing Base analysis");

    let file = std::fs::File::open(&input.path).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

    let tmp = csdemo::Container::parse(&mmap).unwrap();
    let output = csdemo::parser::parse(csdemo::FrameIterator::parse(tmp.inner)).unwrap();

    let header = &output.header;

    tracing::info!("Header: {:?}", header);

    dbg!(&output.player_info);

    for event in output.events.iter() {
        match event {
            csdemo::DemoEvent::Tick(tick) => {}
            csdemo::DemoEvent::ServerInfo(info) => {}
            csdemo::DemoEvent::RankUpdate(update) => {}
            csdemo::DemoEvent::RankReveal(reveal) => {}
            csdemo::DemoEvent::GameEvent(gevent) => {
                match gevent {
                    csdemo::game_event::GameEvent::PlayerTeam(pteam) => {
                        tracing::info!("{:?}", pteam);
                    }
                    csdemo::game_event::GameEvent::RoundOfficiallyEnded(r_end) => {
                        tracing::info!("{:?}", r_end);
                    }
                    csdemo::game_event::GameEvent::PlayerDeath(pdeath) => {
                        tracing::info!("{:?}", pdeath);
                    }
                    other => {}
                };
            }
        };

        /*
        match event.name.as_str() {
            "team_info" => {
                tracing::info!("Team Info: {:?}", event);
            }
            "player_spawn" => {
                // tracing::info!("Player Spawn: {:?}", event);
            }
            "team_score" => {
                tracing::info!("Team Score: {:?}", event);
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
        */
    }

    let map = header.map_name().to_owned();

    BaseInfo {
        map,
        players: Vec::new(),
    }
}
