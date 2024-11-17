pub mod models;
pub mod schema;

mod usersession;
pub use usersession::{UserSession, UserSessionData};

pub mod diesel_sessionstore;

pub mod analysis;

mod gc;

pub async fn db_connection() -> diesel_async::AsyncPgConnection {
    use diesel_async::AsyncConnection;

    let database_url = std::env::var("DATABASE_URL").expect("'DATABASE_URL' must be set");

    diesel_async::AsyncPgConnection::establish(&database_url)
        .await
        .unwrap_or_else(|e| panic!("Error connecting to {} - {:?}", database_url, e))
}

pub mod api;
pub mod steam_api;

pub mod storage;

#[tracing::instrument(skip(storage, steam_api_key))]
pub async fn run_api(
    storage: Box<dyn crate::storage::DemoStorage>,
    steam_api_key: impl Into<String>,
) {
    let session_store = crate::diesel_sessionstore::DieselStore::new();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(time::Duration::hours(
            48,
        )));

    let serve_dir = option_env!("FRONTEND_DIST_DIR").unwrap_or("../frontend/dist/");
    tracing::debug!("Serving static files from {:?}", serve_dir);

    let steam_callback_base_url =
        std::env::var("BASE_URL").unwrap_or("http://localhost:3000".to_owned());
    tracing::debug!("Base-URL: {:?}", steam_callback_base_url);

    let router = axum::Router::new()
        .nest(
            "/api/",
            crate::api::router(crate::api::RouterConfig {
                steam_api_key: steam_api_key.into(),
                steam_callback_base_url,
                steam_callback_path: "/api/steam/callback".into(),
                storage,
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

#[tracing::instrument(skip(storage))]
pub async fn run_analysis(storage: Box<dyn crate::storage::DemoStorage>) {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    loop {
        let mut db_con = db_connection().await;

        let res = crate::analysis::poll_next_task(
            storage.duplicate(),
            &mut db_con,
            move |input: analysis::AnalysisInput, db_con: &mut diesel_async::AsyncPgConnection| {
                Box::pin(async move {
                    let demo_id = input.demoid.clone();

                    let _span = tracing::info_span!("Analysis", demo=?demo_id);

                    tracing::info!("Starting analysis");

                    let mut store_result_fns = Vec::new();
                    for analysis in analysis::ANALYSIS_METHODS.iter().map(|a| a.clone()) {
                        let input = input.clone();
                        let store_result = match tokio::task::spawn_blocking(move || {
                            analysis.analyse(input)
                        })
                        .await
                        {
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

                    let update_process_info = diesel::dsl::update(
                        crate::schema::processing_status::dsl::processing_status,
                    )
                    .set(crate::schema::processing_status::dsl::info.eq(1))
                    .filter(crate::schema::processing_status::dsl::demo_id.eq(demo_id));

                    for store_fn in store_result_fns {
                        store_fn(db_con).await.map_err(|e| ())?;
                    }
                    update_process_info.execute(db_con).await.map_err(|e| ())?;

                    tracing::info!("Completed analysis");

                    Ok::<(), ()>(())
                })
            },
        )
        .await;

        if let Err(e) = res {
            tracing::error!("Polling for next Task: {:?}", e);
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            continue;
        }
    }
}

#[tracing::instrument(skip(storage))]
pub async fn run_garbage_collection(mut storage: Box<dyn crate::storage::DemoStorage>) {
    loop {
        tracing::info!("Running Garbage Collection");

        if let Err(e) = gc::run_gc(storage.as_mut()).await {
            tracing::error!("Running GC {:?}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(15 * 60)).await;
    }
}
