use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: Vec<i64>,
    pub data: serde_json::Value,
    pub expiry_date: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Demo {
    pub steam_id: i64,
    pub demo_id: i64,
}
