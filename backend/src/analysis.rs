use std::path::PathBuf;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub async fn poll_next_task(
    upload_folder: &std::path::Path,
    db_con: &mut diesel_async::pg::AsyncPgConnection,
) -> Result<AnalysisInput, ()> {
    let query = crate::schema::analysis_queue::dsl::analysis_queue
        .order(crate::schema::analysis_queue::dsl::created_at.asc())
        .limit(1)
        .select(crate::models::AnalysisTask::as_select())
        .for_update()
        .skip_locked();

    loop {
        let result = db_con
            .build_transaction()
            .run::<'_, _, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    let mut results: Vec<crate::models::AnalysisTask> = query.load(conn).await?;
                    let final_result = match results.pop() {
                        Some(r) => r,
                        None => return Ok(None),
                    };

                    let delete_query =
                        diesel::dsl::delete(crate::schema::analysis_queue::dsl::analysis_queue)
                            .filter(
                                crate::schema::analysis_queue::dsl::demo_id
                                    .eq(final_result.demo_id),
                            )
                            .filter(
                                crate::schema::analysis_queue::dsl::steam_id
                                    .eq(final_result.steam_id.clone()),
                            );
                    delete_query.execute(conn).await?;

                    Ok(Some(final_result))
                })
            })
            .await;

        match result {
            Ok(Some(r)) => {
                return Ok(AnalysisInput {
                    path: upload_folder
                        .join(&r.steam_id)
                        .join(format!("{}.dem", r.demo_id)),
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

#[derive(Debug, Clone)]
pub struct AnalysisInput {
    pub steamid: String,
    pub demoid: i64,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct BaseInfo {
    pub map: String,
    pub players: Vec<(BasePlayerInfo, BasePlayerStats)>,
}

#[derive(Debug)]
pub struct BasePlayerInfo {
    pub name: String,
    pub steam_id: String,
    pub team: i32,
    pub color: i32,
    pub ingame_id: i32,
}

#[derive(Debug)]
pub struct BasePlayerStats {
    pub kills: usize,
    pub deaths: usize,
    pub damage: usize,
    pub assists: usize,
}

#[tracing::instrument(skip(input))]
pub fn analyse_base(input: AnalysisInput) -> BaseInfo {
    tracing::info!("Performing Base analysis"); 

    let file = std::fs::File::open(&input.path).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

    let result = analysis::endofgame::parse(&mmap).unwrap();

    BaseInfo {
        map: result.map,
        players: result.players.into_iter().map(|(info, stats)| {
            (BasePlayerInfo {
                name: info.name,
                steam_id: info.steam_id,
                team: info.team,
                ingame_id: info.ingame_id,
                color: info.color,
            }, BasePlayerStats {
                    kills: stats.kills,
                    assists: stats.assists,
                    damage: stats.damage,
                    deaths: stats.deaths,
                })
        }).collect()
    }
}

#[tracing::instrument(skip(input))]
pub fn analyse_heatmap(input: AnalysisInput) -> std::collections::HashMap<String, analysis::heatmap::HeatMap> {
    tracing::info!("Generating HEATMAPs");

    let file = std::fs::File::open(&input.path).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };

    let config = analysis::heatmap::Config {
        cell_size: 5.0,
    };
    let (heatmaps, players) = analysis::heatmap::parse(&config, &mmap).unwrap();

    tracing::info!("Got {} Heatmaps", heatmaps.len());
    heatmaps.into_iter().filter_map(|(userid, heatmap)| {
        let player = match players.get(&userid) {
            Some(p) => p,
            None => {
                tracing::warn!("Could not find player: {:?}", userid);
                return None;
            }
        };
        
        Some((player.xuid.to_string(), heatmap))
    }).collect()
}
