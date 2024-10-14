use std::path::PathBuf;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub mod base;
pub mod heatmap;
pub mod perround;

pub trait Analysis {
    fn analyse(
        &self,
        input: AnalysisInput,
    ) -> Result<
        Box<
            dyn FnOnce(
                    &mut diesel_async::pg::AsyncPgConnection,
                ) -> core::pin::Pin<
                    Box<
                        (dyn core::future::Future<Output = Result<(), diesel::result::Error>>
                             + Send
                             + '_),
                    >,
                > + Send,
        >,
        (),
    >;
}

pub static ANALYSIS_METHODS: std::sync::LazyLock<[std::sync::Arc<dyn Analysis + Send + Sync>; 3]> =
    std::sync::LazyLock::new(|| {
        [
            std::sync::Arc::new(base::BaseAnalysis::new()),
            std::sync::Arc::new(heatmap::HeatmapAnalysis::new()),
            std::sync::Arc::new(perround::PerRoundAnalysis::new()),
        ]
    });

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
            .run::<_, diesel::result::Error, _>(|conn| {
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
                                    .eq(final_result.demo_id.clone()),
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
    pub demoid: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct BaseInfo {
    pub map: String,
    pub players: Vec<(BasePlayerInfo, BasePlayerStats)>,
    pub teams: Vec<(u32, BaseTeamInfo)>
}

#[derive(Debug)]
pub struct BaseTeamInfo {
    pub score: usize,
    pub name: String,
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
