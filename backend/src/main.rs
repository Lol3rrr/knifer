use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncPgConnection};

static OPENID: std::sync::LazyLock<steam_openid::SteamOpenId> = std::sync::LazyLock::new(|| {
    steam_openid::SteamOpenId::new("http://192.168.0.156:3000", "/api/steam/callback").unwrap()
});
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
            time::Duration::minutes(15),
        ));

    if !tokio::fs::try_exists(UPLOAD_FOLDER).await.unwrap_or(false) {
        tokio::fs::create_dir_all(UPLOAD_FOLDER).await.unwrap();
    }

    let router = axum::Router::new()
        .nest_service(
            "/api/",
            axum::Router::new()
                .route("/steam/callback", axum::routing::get(steam_callback))
                .route("/steam/login", axum::routing::get(steam_login))
                .route("/demos/upload", axum::routing::post(upload).layer(axum::extract::DefaultBodyLimit::max(1024*1024*500)))
                .route("/demos/list", axum::routing::get(demos_list))
        )
        .layer(session_layer)
        .nest_service("/", tower_http::services::ServeDir::new("frontend/dist/"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn upload(session: backend::UserSession, form: axum::extract::Multipart) -> Result<axum::response::Redirect, (axum::http::StatusCode, &'static str)> {
    let steam_id = session.data().steam_id.ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Not logged in"))?;

    tracing::info!("Upload for Session: {:?}", steam_id);

    let file_content = backend::get_demo_from_upload("demo", form).await.unwrap();

    let user_folder = std::path::Path::new(UPLOAD_FOLDER).join(format!("{}/", steam_id));
    if !tokio::fs::try_exists(&user_folder).await.unwrap_or(false) {
        tokio::fs::create_dir_all(&user_folder).await.unwrap();
    }

    let timestamp_secs = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let demo_file_path = user_folder.join(format!("{}.dem", timestamp_secs));

    tokio::fs::write(demo_file_path, file_content).await.unwrap();

    let query = diesel::dsl::insert_into(backend::schema::demos::dsl::demos).values(backend::models::Demo {
        demo_id: timestamp_secs as i64,
        steam_id: steam_id as i64,
    });
    query.execute(&mut backend::db_connection().await).await.unwrap();

    Ok(axum::response::Redirect::to("/"))
}

async fn steam_login() -> Result<axum::response::Redirect, axum::http::StatusCode> {
    let url = OPENID.get_redirect_url();

    Ok(axum::response::Redirect::to(url))
}

async fn steam_callback(
    mut session: backend::UserSession,
    request: axum::extract::Request,
) -> Result<axum::response::Redirect, axum::http::StatusCode> {
    tracing::info!("Steam Callback");

    let query = request.uri().query().ok_or_else(|| {
        tracing::error!("Missing query in parameters");
        axum::http::StatusCode::BAD_REQUEST
    })?;

    let id = OPENID.verify(query).await.map_err(|e| {
        tracing::error!("Verifying OpenID: {:?}", e);
        axum::http::StatusCode::BAD_REQUEST
    })?;

    session
        .modify_data(|data| {
            data.steam_id = Some(id);
        })
        .await;

    Ok(axum::response::Redirect::to("/"))
}

async fn demos_list(session: backend::UserSession) -> Result<(), axum::http::StatusCode> {
    let steam_id = session.data().steam_id.ok_or_else(|| axum::http::StatusCode::UNAUTHORIZED)?;
    tracing::info!("SteamID: {:?}", steam_id);

    let query = backend::schema::demos::dsl::demos.filter(backend::schema::demos::dsl::steam_id.eq(steam_id as i64));
    let results: Vec<backend::models::Demo> = query.load(&mut backend::db_connection().await).await.unwrap();

    dbg!(&results);

    Ok(())
}
