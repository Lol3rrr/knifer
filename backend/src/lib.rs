pub mod models;
pub mod schema;

mod usersession;
pub use usersession::{UserSession, UserSessionData};

pub mod diesel_sessionstore;

pub mod analysis;

pub async fn db_connection() -> diesel_async::AsyncPgConnection {
    use diesel_async::AsyncConnection;

    let database_url = std::env::var("DATABASE_URL").expect("'DATABASE_URL' must be set");

    diesel_async::AsyncPgConnection::establish(&database_url)
        .await
        .unwrap_or_else(|e| panic!("Error connecting to {} - {:?}", database_url, e))
}

pub async fn get_demo_from_upload(
    name: &str,
    mut form: axum::extract::Multipart,
) -> Option<axum::body::Bytes> {
    while let Ok(field) = form.next_field().await {
        let field = match field {
            Some(f) => f,
            None => continue,
        };

        if field.name().map(|n| n != name).unwrap_or(false) {
            continue;
        }

        if let Ok(data) = field.bytes().await {
            return Some(data);
        }
    }

    None
}

pub mod api;
pub mod steam_api;

#[tracing::instrument(skip(upload_folder, steam_api_key))]
pub async fn run_api(
    upload_folder: impl Into<std::path::PathBuf>,
    steam_api_key: impl Into<String>,
) {
    let upload_folder: std::path::PathBuf = upload_folder.into();

    let session_store = crate::diesel_sessionstore::DieselStore::new();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(time::Duration::hours(
            48,
        )));

    if !tokio::fs::try_exists(&upload_folder).await.unwrap_or(false) {
        tokio::fs::create_dir_all(&upload_folder).await.unwrap();
    }

    let router = axum::Router::new()
        .nest(
            "/api/",
            crate::api::router(crate::api::RouterConfig {
                steam_api_key: steam_api_key.into(),
                steam_callback_base_url: "http://localhost:3000".into(),
                // steam_callback_base_url: "http://localhost:3000".into(),
                steam_callback_path: "/api/steam/callback".into(),
                upload_dir: upload_folder.clone(),
            }),
        )
        .layer(session_layer)
        .nest_service(
            "/",
            tower_http::services::ServeDir::new("../frontend/dist/"),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[tracing::instrument(skip(upload_folder))]
pub async fn run_analysis(upload_folder: impl Into<std::path::PathBuf>) {
    use diesel::prelude::*;
    use diesel_async::{AsyncConnection, RunQueryDsl};

    let upload_folder: std::path::PathBuf = upload_folder.into();

    loop {
        let mut db_con = db_connection().await;
        let input = match crate::analysis::poll_next_task(&upload_folder, &mut db_con).await {
            Ok(i) => i,
            Err(e) => {
                tracing::error!("Polling for next Task: {:?}", e);
                break;
            }
        };

        let demo_id = input.demoid;

        let base_input = input.clone();
        let base_result = tokio::task::spawn_blocking(move || crate::analysis::analyse_base(base_input))
            .await
            .unwrap();

        let heatmap_result = tokio::task::spawn_blocking(move || crate::analysis::analyse_heatmap(input))
            .await
            .unwrap();

        let mut db_con = crate::db_connection().await;

        let (player_info, player_stats): (Vec<_>, Vec<_>) = base_result
            .players
            .into_iter()
            .map(|(info, stats)| {
                (
                    crate::models::DemoPlayer {
                        demo_id,
                        name: info.name,
                        steam_id: info.steam_id.clone(),
                        team: info.team as i16,
                        color: info.color as i16,
                    },
                    crate::models::DemoPlayerStats {
                        demo_id,
                        steam_id: info.steam_id,
                        deaths: stats.deaths as i16,
                        kills: stats.kills as i16,
                        damage: stats.damage as i16,
                        assists: stats.assists as i16,
                    },
                )
            })
            .unzip();

        let player_heatmaps: Vec<_> = heatmap_result.into_iter().map(|(player, heatmap)| {
            tracing::trace!("HeatMap for Player: {:?}", player);

            crate::models::DemoPlayerHeatmap {
                demo_id,
                steam_id: player,
                data: serde_json::to_string(&heatmap).unwrap(),
            }
        }).collect();

        let demo_info = crate::models::DemoInfo {
            demo_id,
            map: base_result.map,
        };

        let store_demo_info_query =
            diesel::dsl::insert_into(crate::schema::demo_info::dsl::demo_info)
                .values(&demo_info)
                .on_conflict(crate::schema::demo_info::dsl::demo_id)
                .do_update()
                .set(
                    crate::schema::demo_info::dsl::map
                        .eq(diesel::upsert::excluded(crate::schema::demo_info::dsl::map)),
                );
        let store_demo_players_query =
            diesel::dsl::insert_into(crate::schema::demo_players::dsl::demo_players)
                .values(player_info)
                .on_conflict_do_nothing();
        let store_demo_player_stats_query =
            diesel::dsl::insert_into(crate::schema::demo_player_stats::dsl::demo_player_stats)
                .values(player_stats)
                .on_conflict((
                    crate::schema::demo_player_stats::dsl::demo_id,
                    crate::schema::demo_player_stats::dsl::steam_id,
                ))
                .do_update()
                .set((
                    crate::schema::demo_player_stats::dsl::deaths.eq(diesel::upsert::excluded(
                        crate::schema::demo_player_stats::dsl::deaths,
                    )),
                    crate::schema::demo_player_stats::dsl::kills.eq(diesel::upsert::excluded(
                        crate::schema::demo_player_stats::dsl::kills,
                    )),
                    crate::schema::demo_player_stats::dsl::assists.eq(diesel::upsert::excluded(
                        crate::schema::demo_player_stats::dsl::assists,
                    )),
                    crate::schema::demo_player_stats::dsl::damage.eq(diesel::upsert::excluded(
                        crate::schema::demo_player_stats::dsl::damage,
                    )),
                ));
        let store_demo_player_heatmaps_query = diesel::dsl::insert_into(crate::schema::demo_heatmaps::dsl::demo_heatmaps)
            .values(player_heatmaps)
            .on_conflict((crate::schema::demo_heatmaps::dsl::demo_id, crate::schema::demo_heatmaps::dsl::steam_id))
            .do_update()
            .set((crate::schema::demo_heatmaps::dsl::data.eq(diesel::upsert::excluded(crate::schema::demo_heatmaps::dsl::data))));
        let update_process_info =
            diesel::dsl::update(crate::schema::processing_status::dsl::processing_status)
                .set(crate::schema::processing_status::dsl::info.eq(1))
                .filter(crate::schema::processing_status::dsl::demo_id.eq(demo_id));

        db_con
            .transaction::<'_, '_, '_, _, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    store_demo_info_query.execute(conn).await?;
                    store_demo_players_query.execute(conn).await?;
                    store_demo_player_stats_query.execute(conn).await?;
                    store_demo_player_heatmaps_query.execute(conn).await?;
                    update_process_info.execute(conn).await?;
                    Ok(())
                })
            })
            .await
            .unwrap();

        tracing::info!("Stored analysis results");
    }
}
