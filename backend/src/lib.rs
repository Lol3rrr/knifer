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

#[tracing::instrument(skip(upload_folder, base_analysis_tx, steam_api_key))]
pub async fn run_api(
    upload_folder: impl Into<std::path::PathBuf>,
    base_analysis_tx: tokio::sync::mpsc::UnboundedSender<analysis::AnalysisInput>,
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
            crate::api::router(
                base_analysis_tx,
                crate::api::RouterConfig {
                    steam_api_key: steam_api_key.into(),
                    steam_callback_base_url: "http://192.168.0.156:3000".into(),
                    steam_callback_path: "/api/steam/callback".into(),
                    upload_dir: upload_folder.clone(),
                },
            ),
        )
        .layer(session_layer)
        .nest_service(
            "/",
            tower_http::services::ServeDir::new("../frontend/dist/"),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[tracing::instrument(skip(base_analysis_rx))]
pub async fn run_analysis(
    mut base_analysis_rx: tokio::sync::mpsc::UnboundedReceiver<analysis::AnalysisInput>,
) {
    use diesel::prelude::*;
    use diesel_async::{AsyncConnection, RunQueryDsl};

    while let Some(input) = base_analysis_rx.recv().await {
        let demo_id = input.demoid;

        let result = tokio::task::spawn_blocking(move || crate::analysis::analyse_base(input))
            .await
            .unwrap();

        tracing::debug!("Analysis-Result: {:?}", result);

        let mut db_con = crate::db_connection().await;

        let store_info_query = diesel::dsl::insert_into(crate::schema::demo_info::dsl::demo_info)
            .values(crate::models::DemoInfo {
                demo_id,
                map: result.map,
            });
        let update_process_info =
            diesel::dsl::update(crate::schema::processing_status::dsl::processing_status)
                .set(crate::schema::processing_status::dsl::info.eq(1))
                .filter(crate::schema::processing_status::dsl::demo_id.eq(demo_id));

        tracing::trace!(?store_info_query, "Store demo info query");
        tracing::trace!(?update_process_info, "Update processing info query");

        db_con
            .transaction::<'_, '_, '_, _, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    store_info_query.execute(conn).await?;
                    update_process_info.execute(conn).await?;
                    Ok(())
                })
            })
            .await
            .unwrap();
    }
}
