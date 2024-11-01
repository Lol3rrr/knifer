use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations =
    diesel_async_migrations::embed_migrations!("../migrations/");

async fn run_migrations(connection: &mut diesel_async::AsyncPgConnection) {
    MIGRATIONS.run_pending_migrations(connection).await.unwrap();
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum CliStorage {
    #[default]
    File,
    S3,
}

#[derive(clap::Parser)]
struct CliArgs {
    #[clap(long = "upload-folder", default_value = "uploads/")]
    upload_folder: std::path::PathBuf,

    #[clap(long = "storage", default_value = "file")]
    storage: CliStorage,

    #[clap(long = "api", default_value_t = true)]
    api: bool,

    #[clap(long = "analysis", default_value_t = true)]
    analysis: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    use clap::Parser;

    let registry = tracing_subscriber::Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::filter_fn(|meta| {
            meta.target().contains("backend") || meta.target().contains("analysis")
        }));
    tracing::subscriber::set_global_default(registry).unwrap();

    tracing::info!("Starting...");

    let args = CliArgs::parse();

    tracing::info!("Applying Migrations");
    run_migrations(&mut backend::db_connection().await).await;
    tracing::info!("Completed Migrations");

    let mut component_set = tokio::task::JoinSet::new();

    let storage: Box<dyn backend::storage::DemoStorage> = match args.storage {
        CliStorage::File => Box::new(backend::storage::FileStorage::new(
            args.upload_folder.clone(),
        )),
        CliStorage::S3 => {
            let credentials = s3::creds::Credentials::from_env_specific(
                Some("S3_ACCESS_KEY"),
                Some("S3_SECRET_KEY"),
                None,
                None,
            )
            .unwrap();

            let region = s3::Region::from_env("S3_REGION", Some("S3_ENDPOINT")).unwrap();

            let bucket =
                std::env::var("S3_BUCKET").expect("Need 'S3_BUCKET' for using s3 storage backend");

            Box::new(backend::storage::S3Storage::new(
                &bucket,
                region,
                credentials,
            ))
        }
    };

    tracing::info!("Starting modules");
    if args.api {
        tracing::info!("Enabled API module");

        let steam_api_key = match std::env::var("STEAM_API_KEY") {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Missing 'STEAM_API_KEY' environment variable - {:?}", e);
                return;
            }
        };

        component_set.spawn(backend::run_api(storage.duplicate(), steam_api_key));
    }
    if args.analysis {
        tracing::info!("Enabled Analysis module");

        component_set.spawn(backend::run_analysis(storage));
    }
    tracing::info!("Started modules");

    component_set.join_all().await;
}
