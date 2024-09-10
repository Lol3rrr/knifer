use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub steamid: Option<String>,
    pub expiry_date: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Demo {
    pub steam_id: String,
    pub demo_id: i64,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub steamid: String,
    pub name: String,
}
