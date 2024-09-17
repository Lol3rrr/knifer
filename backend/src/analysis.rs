use std::path::PathBuf;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub async fn poll_next_task(upload_folder: &std::path::Path, db_con: &mut diesel_async::pg::AsyncPgConnection) -> Result<AnalysisInput, ()> {
    let query = crate::schema::analysis_queue::dsl::analysis_queue.order(crate::schema::analysis_queue::dsl::created_at.asc()).limit(1).select(crate::models::AnalysisTask::as_select()).for_update().skip_locked();

    loop {
        let result = db_con.build_transaction().run::<'_, _, diesel::result::Error, _>(|conn| Box::pin(async move {
            let mut results: Vec<crate::models::AnalysisTask> = query.load(conn).await?;
            let final_result = match results.pop() {
                Some(r) => r,
                None => return Ok(None),
            };

            let delete_query = diesel::dsl::delete(crate::schema::analysis_queue::dsl::analysis_queue).filter(crate::schema::analysis_queue::dsl::demo_id.eq(final_result.demo_id)).filter(crate::schema::analysis_queue::dsl::steam_id.eq(final_result.steam_id.clone()));
            delete_query.execute(conn).await?;

            Ok(Some(final_result))
        })).await;

        match result {
            Ok(Some(r)) => {
                return Ok(AnalysisInput {
                    path: upload_folder.join(&r.steam_id).join(format!("{}.dem", r.demo_id)),
                    steamid: r.steam_id,
                    demoid: r.demo_id,
                });
            }
            Ok(None) => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            Err(e) => {
                tracing::error!("Getting Task from Postgres: {:?}", e);
                return Err(());
            }
        };
    }
}

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
