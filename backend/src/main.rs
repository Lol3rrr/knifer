use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

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

    let (base_analysis_tx, base_analysis_rx) = tokio::sync::mpsc::unbounded_channel::<backend::analysis::AnalysisInput>();

    let mut component_set = tokio::task::JoinSet::new();

    component_set.spawn(backend::run_api(UPLOAD_FOLDER, base_analysis_tx));
    component_set.spawn(backend::run_analysis(base_analysis_rx));

    component_set.join_all().await;
}
