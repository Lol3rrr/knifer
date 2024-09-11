use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

static UPLOAD_FOLDER: &str = "uploads/";

const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations = diesel_async_migrations::embed_migrations!("../migrations/");

async fn run_migrations(connection: &mut diesel_async::AsyncPgConnection) {
    MIGRATIONS.run_pending_migrations(connection).await.unwrap();
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let registry = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::filter_fn(|meta| {
            meta.target().contains("backend")
        }));
    tracing::subscriber::set_global_default(registry).unwrap();

    tracing::info!("Starting...");

    tracing::info!("Applying Migrations");
    run_migrations(&mut backend::db_connection().await).await;
    tracing::info!("Completed Migrations");

    let (base_analysis_tx, mut base_analysis_rx) = tokio::sync::mpsc::unbounded_channel::<backend::analysis::AnalysisInput>();
    tokio::task::spawn_blocking(move || {
        while let Some(input) = base_analysis_rx.blocking_recv() {
            let demo_id = input.demoid;
            let result = backend::analysis::analyse_base(input);

            dbg!(&result);

            let handle = tokio::task::spawn(
                async move {
                    let mut db_con = backend::db_connection().await;
                
                    let store_info_query = diesel::dsl::insert_into(backend::schema::demo_info::dsl::demo_info).values(backend::models::DemoInfo {
                        demo_id,
                        map: result.map,
                    });
                    let update_process_info = diesel::dsl::update(backend::schema::processing_status::dsl::processing_status).set(backend::schema::processing_status::dsl::info.eq(1)).filter(backend::schema::processing_status::dsl::demo_id.eq(demo_id));

                    tracing::trace!(?store_info_query, "Store demo info query");
                    tracing::trace!(?update_process_info, "Update processing info query");

                    store_info_query.execute(&mut db_con).await.unwrap();
                    update_process_info.execute(&mut db_con).await.unwrap();
                }
            );
        }
    });

    let session_store = backend::diesel_sessionstore::DieselStore::new();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(
            time::Duration::hours(48),
        ));

    if !tokio::fs::try_exists(UPLOAD_FOLDER).await.unwrap_or(false) {
        tokio::fs::create_dir_all(UPLOAD_FOLDER).await.unwrap();
    }

    let router = axum::Router::new()
        .nest("/api/", backend::api::router(base_analysis_tx))
        .layer(session_layer)
        .nest_service("/", tower_http::services::ServeDir::new("frontend/dist/"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn demo_info(session: backend::UserSession) -> Result<(), axum::http::StatusCode> {
    Ok(())
}
