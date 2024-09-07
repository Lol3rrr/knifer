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
        .nest("/api/", backend::api::router())
        .layer(session_layer)
        .nest_service("/", tower_http::services::ServeDir::new("frontend/dist/"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn demo_info(session: backend::UserSession) -> Result<(), axum::http::StatusCode> {
    Ok(())
}
