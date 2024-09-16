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
    pub fn new<IS>(api_key: IS) -> Self
    where
        IS: Into<String>,
    {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }

    pub async fn get<T>(&self, path: &str, args: &[(&str, &str)]) -> Result<T, ()>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .http
            .get(path)
            .query(&[("key", &self.api_key)])
            .query(args)
            .send()
            .await
            .map_err(|e| ())?;
        if !response.status().is_success() {
            dbg!(&response);
            return Err(());
        }

        response
            .json::<Response<T>>()
            .await
            .map(|r| r.response)
            .map_err(|e| ())
    }
}
