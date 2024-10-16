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

    let serve_dir = option_env!("FRONTEND_DIST_DIR").unwrap_or("../frontend/dist/");
    tracing::debug!("Serving static files from {:?}", serve_dir);

    let steam_callback_base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:3000".to_owned());
    tracing::debug!("Base-URL: {:?}", steam_callback_base_url);

    let router = axum::Router::new()
        .nest(
            "/api/",
            crate::api::router(crate::api::RouterConfig {
                steam_api_key: steam_api_key.into(),
                steam_callback_base_url,
                steam_callback_path: "/api/steam/callback".into(),
                upload_dir: upload_folder.clone(),
            }),
        )
        .layer(session_layer)
        .nest_service("/", tower_http::services::ServeDir::new(serve_dir));

    let listen_addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        3000,
    );
    tracing::info!("Listening on Addr: {:?}", listen_addr);

    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
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

        let demo_id = input.demoid.clone();

        let mut store_result_fns = Vec::new();
        for analysis in analysis::ANALYSIS_METHODS.iter().map(|a| a.clone()) {
            let input = input.clone();
            let store_result =
                match tokio::task::spawn_blocking(move || analysis.analyse(input)).await {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        tracing::error!("Analysis failed: {:?}", e);
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("Joining Task: {:?}", e);
                        continue;
                    }
                };

            store_result_fns.push(store_result);
        }

        let mut db_con = crate::db_connection().await;

        let update_process_info =
            diesel::dsl::update(crate::schema::processing_status::dsl::processing_status)
                .set(crate::schema::processing_status::dsl::info.eq(1))
                .filter(crate::schema::processing_status::dsl::demo_id.eq(demo_id));

        db_con
            .transaction::<'_, '_, '_, _, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    for store_fn in store_result_fns {
                        store_fn(conn).await?;
                    }
                    update_process_info.execute(conn).await?;

                    Ok(())
                })
            })
            .await
            .unwrap();

        tracing::info!("Stored analysis results");
    }
}
