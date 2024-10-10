-- Your SQL goes here
CREATE TABLE IF NOT EXISTS ANALYSIS_QUEUE (
  demo_id TEXT PRIMARY KEY,
  steam_id Text NOT NULL,
  created_at timestamp NOT NULL default current_timestamp
);
