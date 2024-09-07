use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserSessionData {
    pub steam_id: Option<u64>,
}

impl Default for UserSessionData {
    fn default() -> Self {
        Self { steam_id: None }
    }
}

pub struct UserSession {
    pub session: tower_sessions::Session,
    data: UserSessionData,
}

impl UserSession {
    const KEY: &'static str = "user.data";

    pub fn data(&self) -> &UserSessionData {
        &self.data
    }

    pub async fn modify_data<F>(&mut self, func: F)
    where
        F: FnOnce(&mut UserSessionData),
    {
        let mut entry = &mut self.data;
        func(&mut entry);

        self.session.insert(Self::KEY, entry).await.unwrap();
    }
}

#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for UserSession
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        req: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = tower_sessions::Session::from_request_parts(req, state).await?;

        let guest_data: UserSessionData = session.get(Self::KEY).await.unwrap().unwrap_or_default();

        Ok(Self {
            session,
            data: guest_data,
        })
    }
}
