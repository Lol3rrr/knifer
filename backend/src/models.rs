use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub steamid: Option<String>,
    pub expiry_date: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::demos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDemo {
    pub steam_id: String,
    pub demo_id: String,
}

#[derive(Selectable, Queryable, Debug)]
#[diesel(table_name = crate::schema::demos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Demo {
    pub steam_id: String,
    pub demo_id: String,
    pub uploaded_at: diesel::data_types::PgTimestamp,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub steamid: String,
    pub name: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demo_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DemoInfo {
    pub demo_id: String,
    pub map: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demo_players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DemoPlayer {
    pub demo_id: String,
    pub steam_id: String,
    pub name: String,
    pub team: i16,
    pub color: i16,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demo_player_stats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DemoPlayerStats {
    pub demo_id: String,
    pub steam_id: String,
    pub kills: i16,
    pub deaths: i16,
    pub damage: i16,
    pub assists: i16,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demo_teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DemoTeam {
    pub demo_id: String,
    pub team: i16,
    pub end_score: i16,
    pub start_name: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::processing_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProcessingStatus {
    pub demo_id: String,
    pub info: i16,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::analysis_queue)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AddAnalysisTask {
    pub demo_id: String,
    pub steam_id: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::analysis_queue)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AnalysisTask {
    pub demo_id: String,
    pub steam_id: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demo_heatmaps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DemoPlayerHeatmap {
    pub demo_id: String,
    pub steam_id: String,
    pub team: String,
    pub data: String,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::demo_round)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DemoRound {
    pub demo_id: String,
    pub round_number: i16,
    pub start_tick: i64,
    pub end_tick: i64,
    pub win_reason: String,
    pub events: serde_json::Value,
}
