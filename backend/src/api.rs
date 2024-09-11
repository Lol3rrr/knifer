pub mod demos {
    use crate::UserSession;
    use diesel_async::RunQueryDsl;
    use diesel::prelude::*;
    use axum::extract::{State, Path};
    use std::sync::Arc;

    struct DemoState {
        upload_folder: std::path::PathBuf,
        base_analysis: tokio::sync::mpsc::UnboundedSender<crate::analysis::AnalysisInput>
    }

    pub fn router<P>(upload_folder: P, base_analysis: tokio::sync::mpsc::UnboundedSender<crate::analysis::AnalysisInput>) -> axum::Router where P: Into<std::path::PathBuf> {
        axum::Router::new()
            .route("/list", axum::routing::get(list))
            .route("/upload", axum::routing::post(upload).layer(axum::extract::DefaultBodyLimit::max(500*1024*1024)))
            .route("/:id/info", axum::routing::get(info))
            .with_state(Arc::new(DemoState {
                upload_folder: upload_folder.into(),
                base_analysis,
            }))
    }

    #[tracing::instrument(skip(session))]
    async fn list(session: UserSession) -> Result<axum::response::Json<Vec<common::BaseDemoInfo>>, axum::http::StatusCode> {
        let steam_id = session.data().steam_id.ok_or_else(|| axum::http::StatusCode::UNAUTHORIZED)?;
        tracing::info!("SteamID: {:?}", steam_id);

        let query = crate::schema::demos::dsl::demos.inner_join(crate::schema::demo_info::dsl::demo_info).select((crate::models::Demo::as_select(), crate::models::DemoInfo::as_select())).filter(crate::schema::demos::dsl::steam_id.eq(steam_id.to_string()));
        let results: Vec<(crate::models::Demo, crate::models::DemoInfo)> = query.load(&mut crate::db_connection().await).await.unwrap();
    
        Ok(axum::response::Json(results.into_iter().map(|(demo, info)| common::BaseDemoInfo {
            id: demo.demo_id,
            map: info.map,
        }).collect::<Vec<_>>()))
    }

    #[tracing::instrument(skip(state, session))]
    async fn upload(State(state): State<Arc<DemoState>>, session: crate::UserSession, form: axum::extract::Multipart) -> Result<axum::response::Redirect, (axum::http::StatusCode, &'static str)> {
        let steam_id = session.data().steam_id.ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Not logged in"))?;

        tracing::info!("Upload for Session: {:?}", steam_id);

        let file_content = crate::get_demo_from_upload("demo", form).await.unwrap();

        let user_folder = std::path::Path::new(&state.upload_folder).join(format!("{}/", steam_id));
        if !tokio::fs::try_exists(&user_folder).await.unwrap_or(false) {
            tokio::fs::create_dir_all(&user_folder).await.unwrap();
        }

        let timestamp_secs = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let demo_id = timestamp_secs as i64;
        let demo_file_path = user_folder.join(format!("{}.dem", timestamp_secs));

        tokio::fs::write(&demo_file_path, file_content).await.unwrap();

        let mut db_con = crate::db_connection().await;

        let query = diesel::dsl::insert_into(crate::schema::demos::dsl::demos).values(crate::models::Demo {
            demo_id,
            steam_id: steam_id.to_string(),
        });
        query.execute(&mut db_con).await.unwrap();

        state.base_analysis.send(crate::analysis::AnalysisInput {
            steamid: steam_id.to_string(),
            demoid: demo_id,
            path: demo_file_path,
        });
        let processing_query = diesel::dsl::insert_into(crate::schema::processing_status::dsl::processing_status).values(crate::models::ProcessingStatus {
            demo_id,
            info: 0,
        });
        processing_query.execute(&mut db_con).await.unwrap();

        Ok(axum::response::Redirect::to("/"))
    }

    #[tracing::instrument(skip(session))]
    async fn info(session: UserSession, Path(demo_id): Path<i64>) -> Result<(), axum::http::StatusCode> {
        tracing::info!("Get info for Demo: {:?}", demo_id);

        Ok(())
    }
}

pub mod steam {
    use axum::extract::State;
    use serde::Deserialize;
    use std::{sync::Arc, collections::HashMap};
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    #[derive(Debug, Deserialize)]
    struct ProfileInfoResponse {
        players: Vec<ProfileInfo>
    }
    #[derive(Debug, Deserialize)]
    struct ProfileInfo {
        pub steamid: String,
        pub personaname: String,
        #[serde(flatten)]
        other: HashMap<String, serde_json::Value>,
    }

    pub fn router(url: &str, callback_path: &str) -> axum::Router {
        axum::Router::new()
            .route("/login", axum::routing::get(steam_login))
            .route("/callback", axum::routing::get(steam_callback))
            .with_state(Arc::new(steam_openid::SteamOpenId::new(url, callback_path).unwrap()))
    }

    #[tracing::instrument(skip(openid))]
    async fn steam_login(State(openid): State<Arc<steam_openid::SteamOpenId>>) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        let url = openid.get_redirect_url();

        Ok(axum::response::Redirect::to(url))
    }

    #[tracing::instrument(skip(openid, session, request))]
    async fn steam_callback(
        State(openid): State<Arc<steam_openid::SteamOpenId>>,
        mut session: crate::UserSession,
        request: axum::extract::Request,
    ) -> Result<axum::response::Redirect, axum::http::StatusCode> {
        tracing::info!("Steam Callback");

        let query = request.uri().query().ok_or_else(|| {
            tracing::error!("Missing query in parameters");
            axum::http::StatusCode::BAD_REQUEST
        })?;

        let id = openid.verify(query).await.map_err(|e| {
            tracing::error!("Verifying OpenID: {:?}", e);
            axum::http::StatusCode::BAD_REQUEST
        })?;

        let steam_client = crate::steam_api::Client::new(std::env::var("STEAM_API_KEY").unwrap());
        let profile_response_data: ProfileInfoResponse = match steam_client.get("http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/", &[("steamids", &format!("{}", id))]).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Getting Steam Profile Info: {:?}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let mut db_con = crate::db_connection().await;
        for player in profile_response_data.players {
            let query = diesel::dsl::insert_into(crate::schema::users::dsl::users).values(crate::models::User {
                steamid: player.steamid,
                name: player.personaname.clone(),
            }).on_conflict(crate::schema::users::dsl::steamid).do_update().set((crate::schema::users::dsl::name.eq(player.personaname)));
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
        axum::Router::new()
            .route("/status", axum::routing::get(status))
    }

    #[tracing::instrument(skip(session))]
    async fn status(session: crate::UserSession) -> Result<axum::response::Json<common::UserStatus>, reqwest::StatusCode> {
        let steam_id = match session.data().steam_id {
            Some(s) => s,
            None => {
                return Err(axum::http::StatusCode::UNAUTHORIZED);
            }
        };

        tracing::info!("Load user info");

        let mut db_con = crate::db_connection().await;

        let query = crate::schema::users::dsl::users.filter(crate::schema::users::dsl::steamid.eq(format!("{}", steam_id)));

        let mut result = query.load::<crate::models::User>(&mut db_con).await.unwrap();
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

pub fn router(base_analysis: tokio::sync::mpsc::UnboundedSender<crate::analysis::AnalysisInput>) -> axum::Router {
    axum::Router::new()
        .nest("/steam/", steam::router("http://localhost:3000", "/api/steam/callback"))
        .nest("/demos/", demos::router("uploads/", base_analysis))
        .nest("/user/", user::router())
}
