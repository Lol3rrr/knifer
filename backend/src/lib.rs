pub mod models;
pub mod schema;

mod usersession;
pub use usersession::{UserSessionData, UserSession};

pub mod diesel_sessionstore;

pub mod analysis;

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
pub mod steam_api {
    use serde::Deserialize;

    pub struct Client {
        http: reqwest::Client,
        api_key: String,
    }

    #[derive(Debug, Deserialize)]
    struct Response<T> {
        response: T,
    }

    impl Client {
        pub fn new<IS>(api_key: IS) -> Self where IS: Into<String> {
            Self {
                http: reqwest::Client::new(),
                api_key: api_key.into(),
            }
        }

        pub async fn get<T>(&self, path: &str, args: &[(&str, &str)]) -> Result<T, ()> where T: serde::de::DeserializeOwned {
            let response = self.http.get(path).query(&[("key", &self.api_key)]).query(args).send().await.map_err(|e| ())?;
            if !response.status().is_success() {
                dbg!(&response);
                return Err(());
            }

            response.json::<Response<T>>().await.map(|r| r.response).map_err(|e| ())
        }
    }
}
