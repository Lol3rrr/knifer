use crate::UserSession;
use axum::extract::{Path, State};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

struct DemoState {
    upload_folder: std::path::PathBuf,
}

pub fn router<P>(upload_folder: P) -> axum::Router
where
    P: Into<std::path::PathBuf>,
{
    axum::Router::new()
        .route("/list", axum::routing::get(list))
        .route(
            "/upload",
            axum::routing::post(upload)
                .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024)),
        )
        .route("/:id/info", axum::routing::get(info))
        .route("/:id/analysis/scoreboard", axum::routing::get(scoreboard))
        .route("/:id/reanalyse", axum::routing::get(analyise))
        .with_state(Arc::new(DemoState {
            upload_folder: upload_folder.into(),
        }))
}

#[tracing::instrument(skip(session))]
async fn list(
    session: UserSession,
) -> Result<axum::response::Json<Vec<common::BaseDemoInfo>>, axum::http::StatusCode> {
    let steam_id = session
        .data()
        .steam_id
        .ok_or_else(|| axum::http::StatusCode::UNAUTHORIZED)?;
    tracing::info!("SteamID: {:?}", steam_id);

    let query = crate::schema::demos::dsl::demos
        .inner_join(crate::schema::demo_info::dsl::demo_info)
        .select((
            crate::models::Demo::as_select(),
            crate::models::DemoInfo::as_select(),
        ))
        .filter(crate::schema::demos::dsl::steam_id.eq(steam_id.to_string()));
    let results: Vec<(crate::models::Demo, crate::models::DemoInfo)> =
        query.load(&mut crate::db_connection().await).await.unwrap();

    Ok(axum::response::Json(
        results
            .into_iter()
            .map(|(demo, info)| common::BaseDemoInfo {
                id: demo.demo_id,
                map: info.map,
            })
            .collect::<Vec<_>>(),
    ))
}

#[tracing::instrument(skip(state, session))]
async fn upload(
    State(state): State<Arc<DemoState>>,
    session: crate::UserSession,
    form: axum::extract::Multipart,
) -> Result<axum::response::Redirect, (axum::http::StatusCode, &'static str)> {
    let steam_id = session
        .data()
        .steam_id
        .ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Not logged in"))?;

    tracing::info!("Upload for Session: {:?}", steam_id);

    let file_content = crate::get_demo_from_upload("demo", form).await.unwrap();

    let user_folder = std::path::Path::new(&state.upload_folder).join(format!("{}/", steam_id));
    if !tokio::fs::try_exists(&user_folder).await.unwrap_or(false) {
        tokio::fs::create_dir_all(&user_folder).await.unwrap();
    }

    let timestamp_secs = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let demo_id = timestamp_secs as i64;
    let demo_file_path = user_folder.join(format!("{}.dem", timestamp_secs));

    tokio::fs::write(&demo_file_path, file_content)
        .await
        .unwrap();

    let mut db_con = crate::db_connection().await;

    // Turn all of this into a single transaction

    let query =
        diesel::dsl::insert_into(crate::schema::demos::dsl::demos).values(crate::models::Demo {
            demo_id,
            steam_id: steam_id.to_string(),
        });
    query.execute(&mut db_con).await.unwrap();

    let queue_query = diesel::dsl::insert_into(crate::schema::analysis_queue::dsl::analysis_queue)
        .values(crate::models::AddAnalysisTask {
            demo_id,
            steam_id: steam_id.to_string(),
        });
    queue_query.execute(&mut db_con).await.unwrap();

    let processing_query =
        diesel::dsl::insert_into(crate::schema::processing_status::dsl::processing_status)
            .values(crate::models::ProcessingStatus { demo_id, info: 0 });
    processing_query.execute(&mut db_con).await.unwrap();

    Ok(axum::response::Redirect::to("/"))
}

#[tracing::instrument(skip(session))]
async fn analyise(
    session: crate::UserSession,
    Path(demo_id): Path<i64>,
) -> Result<(), (axum::http::StatusCode, &'static str)> {
    let steam_id = session
        .data()
        .steam_id
        .ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Not logged in"))?;

    tracing::info!("Upload for Session: {:?}", steam_id);

    let mut db_con = crate::db_connection().await;

    let query = crate::schema::demos::dsl::demos
        .filter(crate::schema::demos::dsl::steam_id.eq(steam_id.to_string()))
        .filter(crate::schema::demos::dsl::demo_id.eq(demo_id));
    let result: Vec<_> = query
        .load::<crate::models::Demo>(&mut db_con)
        .await
        .unwrap();

    if result.len() != 1 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "Expected exactly 1 demo to match",
        ));
    }

    let queue_query = diesel::dsl::insert_into(crate::schema::analysis_queue::dsl::analysis_queue)
        .values(crate::models::AddAnalysisTask {
            demo_id,
            steam_id: steam_id.to_string(),
        });
    queue_query.execute(&mut db_con).await.unwrap();

    Ok(())
}

#[tracing::instrument(skip(_session))]
async fn info(
    _session: UserSession,
    Path(demo_id): Path<i64>,
) -> Result<axum::response::Json<common::DemoInfo>, axum::http::StatusCode> {
    tracing::info!("Get info for Demo: {:?}", demo_id);

    let query = crate::schema::demo_info::dsl::demo_info
        .select(crate::models::DemoInfo::as_select())
        .filter(crate::schema::demo_info::dsl::demo_id.eq(demo_id));
    let mut results: Vec<crate::models::DemoInfo> =
        query.load(&mut crate::db_connection().await).await.unwrap();

    if results.len() != 1 {
        tracing::error!("Expected only 1 match but got {} matches", results.len());
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let result = results.pop().unwrap();

    Ok(axum::Json(common::DemoInfo {
        id: result.demo_id,
        map: result.map,
    }))
}

#[tracing::instrument(skip(session))]
async fn scoreboard(
    session: UserSession,
    Path(demo_id): Path<i64>,
) -> Result<axum::response::Json<common::demo_analysis::ScoreBoard>, axum::http::StatusCode> {
    let query = crate::schema::demo_players::dsl::demo_players
        .inner_join(
            crate::schema::demo_player_stats::dsl::demo_player_stats.on(
                crate::schema::demo_players::dsl::demo_id
                    .eq(crate::schema::demo_player_stats::dsl::demo_id)
                    .and(
                        crate::schema::demo_players::dsl::steam_id
                            .eq(crate::schema::demo_player_stats::dsl::steam_id),
                    ),
            ),
        )
        .filter(crate::schema::demo_players::dsl::demo_id.eq(demo_id));

    let mut db_con = crate::db_connection().await;

    let response: Vec<(crate::models::DemoPlayer, crate::models::DemoPlayerStats)> =
        match query.load(&mut db_con).await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Querying DB: {:?}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    if response.is_empty() {
        tracing::error!("DB Response was empty");
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    let team1_number: i16 = response.last().map(|(p, _)| p.team).unwrap();

    let mut team1 = Vec::new();
    let mut team2 = Vec::new();
    for (player, stats) in response {
        let team_vec = if player.team == team1_number {
            &mut team1
        } else {
            &mut team2
        };

        team_vec.push(common::demo_analysis::ScoreBoardPlayer {
            name: player.name,
            kills: stats.kills as usize,
            deaths: stats.deaths as usize,
            damage: stats.damage as usize,
            assists: stats.assists as usize,
        });
    }

    Ok(axum::Json(common::demo_analysis::ScoreBoard {
        team1,
        team2,
    }))
}
