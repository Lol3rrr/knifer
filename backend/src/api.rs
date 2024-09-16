pub mod demos;

pub mod steam {
    use axum::extract::State;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use serde::Deserialize;
    use std::{collections::HashMap, sync::Arc};

    #[derive(Debug, Deserialize)]
    struct ProfileInfoResponse {
        players: Vec<ProfileInfo>,
    }
    #[derive(Debug, Deserialize)]
    struct ProfileInfo {
        pub steamid: String,
        pub personaname: String,
        #[serde(flatten)]
        _other: HashMap<String, serde_json::Value>,
    }

    struct SteamApiState {
        openid: steam_openid::SteamOpenId,
        api_key: String,
    }

    pub fn router(url: &str, callback_path: &str, api_key: impl Into<String>) -> axum::Router {
        axum::Router::new()
            .route("/login", axum::routing::get(steam_login))
            .route("/callback", axum::routing::get(steam_callback))
            .with_state(Arc::new(SteamApiState {
                openid: steam_openid::SteamOpenId::new(url, callback_path).unwrap(),
                api_key: api_key.into(),
            }))
    }

    #[tracing::instrument(skip(state))]
    async fn steam_login(
        State(state): State<Arc<SteamApiState>>,
    ) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        let url = state.openid.get_redirect_url();

        Ok(axum::response::Redirect::to(url))
    }

    #[tracing::instrument(skip(state, session, request))]
    async fn steam_callback(
        State(state): State<Arc<SteamApiState>>,
        mut session: crate::UserSession,
        request: axum::extract::Request,
    ) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        tracing::info!("Steam Callback");

        let query = request.uri().query().ok_or_else(|| {
            tracing::error!("Missing query in parameters");
            axum::http::StatusCode::BAD_REQUEST
        })?;

        let id = state.openid.verify(query).await.map_err(|e| {
            tracing::error!("Verifying OpenID: {:?}", e);
            axum::http::StatusCode::BAD_REQUEST
        })?;

        let steam_client = crate::steam_api::Client::new(&state.api_key);
        let profile_response_data: ProfileInfoResponse = match steam_client
            .get(
                "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/",
                &[("steamids", &format!("{}", id))],
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Getting Steam Profile Info: {:?}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut db_con = crate::db_connection().await;
        for player in profile_response_data.players {
            let query = diesel::dsl::insert_into(crate::schema::users::dsl::users)
                .values(crate::models::User {
                    steamid: player.steamid,
                    name: player.personaname.clone(),
                })
                .on_conflict(crate::schema::users::dsl::steamid)
                .do_update()
                .set(crate::schema::users::dsl::name.eq(player.personaname));
            tracing::debug!("Running Query: {:?}", query);

            if let Err(e) = query.execute(&mut db_con).await {
                tracing::error!("Inserting/Updating user steam info: {:?}", e);
            }
        }

        session
            .modify_data(|data| {
                data.steam_id = Some(id);
            })
            .await;

        Ok(axum::response::Redirect::to("/"))
    }
}

pub mod user {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    pub fn router() -> axum::Router {
        axum::Router::new().route("/status", axum::routing::get(status))
    }

    #[tracing::instrument(skip(session))]
    async fn status(
        session: crate::UserSession,
    ) -> Result<axum::response::Json<common::UserStatus>, reqwest::StatusCode> {
        let steam_id = match session.data().steam_id {
            Some(s) => s,
            None => {
                return Err(axum::http::StatusCode::UNAUTHORIZED);
            }
        };

        tracing::info!("Load user info");

        let mut db_con = crate::db_connection().await;

        let query = crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::steamid.eq(format!("{}", steam_id)));

        let mut result = query
            .load::<crate::models::User>(&mut db_con)
            .await
            .unwrap();
        if result.len() != 1 {
            tracing::error!("Unexpected query result: {:?}", result);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }

        let user_entry = result.pop().unwrap();

        Ok(axum::Json(common::UserStatus {
            name: user_entry.name,
            steamid: user_entry.steamid,
        }))
    }
}

pub struct RouterConfig {
    pub steam_api_key: String,
    pub steam_callback_base_url: String,
    pub steam_callback_path: String,
    pub upload_dir: std::path::PathBuf,
}

pub fn router(
    base_analysis: tokio::sync::mpsc::UnboundedSender<crate::analysis::AnalysisInput>,
    config: RouterConfig,
) -> axum::Router {
    axum::Router::new()
        .nest(
            "/steam/",
            steam::router(
                &config.steam_callback_base_url,
                &config.steam_callback_path,
                config.steam_api_key,
            ),
        )
        .nest("/demos/", demos::router(config.upload_dir, base_analysis))
        .nest("/user/", user::router())
}
