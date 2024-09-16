use std::collections::HashMap;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Debug, Clone)]
pub struct DieselStore {}

static EXPIRY_FORMAT: std::sync::LazyLock<
    &[time::format_description::BorrowedFormatItem<'static>],
> = std::sync::LazyLock::new(|| {
    time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
        )
});

impl DieselStore {
    pub fn new() -> Self {
        Self {}
    }

    fn expiry_to_string(&self, expiry_date: &time::OffsetDateTime) -> String {
        expiry_date.format(&EXPIRY_FORMAT).unwrap()
    }
    fn string_to_expiry(&self, input: &str) -> time::OffsetDateTime {
        time::OffsetDateTime::parse(input, &EXPIRY_FORMAT).unwrap()
    }
}

#[async_trait::async_trait]
impl tower_sessions::SessionStore for DieselStore {
    async fn save(
        &self,
        session_record: &tower_sessions::session::Record,
    ) -> tower_sessions::session_store::Result<()> {
        let db_id = session_record.id.0.to_string();

        let expiry_date = self.expiry_to_string(&session_record.expiry_date);

        let steamid = session_record
            .data
            .get(crate::UserSession::KEY)
            .map(|e| serde_json::from_value::<crate::UserSessionData>(e.clone()).ok())
            .flatten()
            .map(|d| d.steam_id.map(|s| s.to_string()))
            .flatten();

        let query = diesel::dsl::insert_into(crate::schema::sessions::dsl::sessions)
            .values(crate::models::Session {
                id: db_id,
                steamid: steamid.clone(),
                expiry_date: expiry_date.clone(),
            })
            .on_conflict(crate::schema::sessions::dsl::id)
            .do_update()
            .set((
                crate::schema::sessions::dsl::steamid.eq(steamid),
                crate::schema::sessions::dsl::expiry_date.eq(expiry_date),
            ));

        let mut connection = crate::db_connection().await;

        query.execute(&mut connection).await.unwrap();

        Ok(())
    }

    async fn load(
        &self,
        session_id: &tower_sessions::session::Id,
    ) -> tower_sessions::session_store::Result<Option<tower_sessions::session::Record>> {
        let db_id = session_id.0.to_string();

        let query = crate::schema::sessions::dsl::sessions
            .filter(crate::schema::sessions::dsl::id.eq(db_id));

        let mut connection = crate::db_connection().await;

        let mut result: Vec<crate::models::Session> = query.load(&mut connection).await.unwrap();

        if result.len() > 1 {
            tracing::error!("Found more than 1 result");
            return Err(tower_sessions::session_store::Error::Backend(
                "Found more than 1 result".to_string(),
            ));
        }

        if result.is_empty() {
            return Ok(None);
        }

        let result = result.pop().unwrap();

        let data = {
            let mut tmp = HashMap::<String, _>::new();
            tmp.insert(
                crate::UserSession::KEY.to_string(),
                serde_json::to_value(&crate::UserSessionData {
                    steam_id: result.steamid.map(|s| s.parse().ok()).flatten(),
                })
                .unwrap(),
            );
            tmp
        };

        Ok(Some(tower_sessions::session::Record {
            id: tower_sessions::session::Id(result.id.parse().unwrap()),
            data,
            expiry_date: self.string_to_expiry(&result.expiry_date),
        }))
    }

    async fn delete(
        &self,
        session_id: &tower_sessions::session::Id,
    ) -> tower_sessions::session_store::Result<()> {
        let db_id = session_id.0.to_string();

        let query = crate::schema::sessions::dsl::sessions
            .filter(crate::schema::sessions::dsl::id.eq(db_id));

        let mut connection = crate::db_connection().await;
        diesel::dsl::delete(query)
            .execute(&mut connection)
            .await
            .unwrap();

        Ok(())
    }
}
