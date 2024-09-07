pub mod models;
pub mod schema;

mod usersession;
pub use usersession::{UserSessionData, UserSession};

pub mod diesel_sessionstore;

pub async fn db_connection() -> diesel_async::AsyncPgConnection {
    use diesel_async::AsyncConnection;

    let database_url = std::env::var("DATABASE_URL").expect("'DATABASE_URL' must be set");

    diesel_async::AsyncPgConnection::establish(&database_url).await.unwrap_or_else(|e| panic!("Error connecting to {} - {:?}", database_url, e))
}

pub async fn get_demo_from_upload(name: &str, mut form: axum::extract::Multipart) -> Option<axum::body::Bytes> {
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
