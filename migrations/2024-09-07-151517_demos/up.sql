-- Your SQL goes here
CREATE TABLE IF NOT EXISTS demos (
  steam_id TEXT NOT NULL,
  demo_id bigint NOT NULL,
  PRIMARY KEY(steam_id, demo_id)
)
