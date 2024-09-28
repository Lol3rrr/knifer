use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations =
    diesel_async_migrations::embed_migrations!("../migrations/");

async fn run_migrations(connection: &mut diesel_async::AsyncPgConnection) {
    MIGRATIONS.run_pending_migrations(connection).await.unwrap();
}

#[derive(clap::Parser)]
struct CliArgs {
    #[clap(long = "upload-folder", default_value = "uploads/")]
    upload_folder: std::path::PathBuf,

    #[clap(long = "api", default_value_t = true)]
    api: bool,

    #[clap(long = "analysis", default_value_t = true)]
    analysis: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    use clap::Parser;

    let registry = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::filter_fn(|meta| {
            meta.target().contains("backend")
            || meta.target().contains("analysis")
        }));
    tracing::subscriber::set_global_default(registry).unwrap();

    tracing::info!("Starting...");

    let args = CliArgs::parse();

    tracing::info!("Applying Migrations");
    run_migrations(&mut backend::db_connection().await).await;
    tracing::info!("Completed Migrations");

    let mut component_set = tokio::task::JoinSet::new();

    if args.api {
        let steam_api_key = match std::env::var("STEAM_API_KEY") {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Missing 'STEAM_API_KEY' environment variable - {:?}", e);
                return;
            }
        };

        component_set.spawn(backend::run_api(args.upload_folder.clone(), steam_api_key));
    }
    if args.analysis {
        component_set.spawn(backend::run_analysis(args.upload_folder.clone()));
    }

    component_set.join_all().await;
}
