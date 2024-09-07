use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Debug, Clone)]
pub struct DieselStore {}

static EXPIRY_FORMAT: std::sync::LazyLock<&[time::format_description::BorrowedFormatItem<'static>]> = std::sync::LazyLock::new(|| {
        time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
        )
    });

impl DieselStore {

    pub fn new() -> Self {
        Self {}
    }

    fn id_to_bytes(&self, val: i128) -> Vec<i64> {
        let id_bytes = val.to_be_bytes();
        vec![i64::from_be_bytes((id_bytes[0..8]).try_into().unwrap()), i64::from_be_bytes((id_bytes[8..16]).try_into().unwrap())]
    }
    fn bytes_to_id(&self, val: Vec<i64>) -> i128 {
        assert_eq!(2, val.len());

        let fb = val[0].to_be_bytes();
        let sb = val[1].to_be_bytes();

        i128::from_be_bytes([fb[0], fb[1], fb[2], fb[3], fb[4], fb[5], fb[6], fb[7], sb[0], sb[1], sb[2], sb[3], sb[4], sb[5], sb[6], sb[7]])
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
    async fn save(&self,session_record: &tower_sessions::session::Record) ->  tower_sessions::session_store::Result<()> { 
        let db_id = self.id_to_bytes(session_record.id.0);

        let data = serde_json::to_value(&session_record.data).unwrap();
        let expiry_date = self.expiry_to_string(&session_record.expiry_date);

        let query = diesel::dsl::insert_into(crate::schema::sessions::dsl::sessions)
            .values(crate::models::Session {
                id: db_id,
                data: data.clone(),
                expiry_date: expiry_date.clone(),
            })
            .on_conflict(crate::schema::sessions::dsl::id)
            .do_update()
            .set((crate::schema::sessions::dsl::data.eq(data), crate::schema::sessions::dsl::expiry_date.eq(expiry_date)));

        let mut connection = crate::db_connection().await;

        query.execute(&mut connection).await.unwrap();

        Ok(())
    }

    async fn load(&self,session_id: &tower_sessions::session::Id) ->  tower_sessions::session_store::Result<Option<tower_sessions::session::Record>> {
        let db_id = self.id_to_bytes(session_id.0);

        let query = crate::schema::sessions::dsl::sessions.filter(crate::schema::sessions::dsl::id.eq(db_id));

        let mut connection = crate::db_connection().await;

        let mut result: Vec<crate::models::Session> = query.load(&mut connection).await.unwrap();

        if result.len() > 1 {
            tracing::error!("Found more than 1 result");
            return Err(tower_sessions::session_store::Error::Backend("Found more than 1 result".to_string()));
        }

        let result = result.pop().unwrap();

        Ok(Some(tower_sessions::session::Record {
            id: tower_sessions::session::Id(self.bytes_to_id(result.id)),
            data: serde_json::from_value(result.data).unwrap(),
            expiry_date: self.string_to_expiry(&result.expiry_date),
        }))
    }

    async fn delete(&self,session_id: &tower_sessions::session::Id) -> tower_sessions::session_store::Result<()> {
        let db_id = self.id_to_bytes(session_id.0);

        let query = crate::schema::sessions::dsl::sessions.filter(crate::schema::sessions::dsl::id.eq(db_id));

        let mut connection = crate::db_connection().await;
        diesel::dsl::delete(query).execute(&mut connection).await.unwrap();

        Ok(())
    }
}
