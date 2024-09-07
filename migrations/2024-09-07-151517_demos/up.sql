-- Your SQL goes here
CREATE TABLE IF NOT EXISTS demos (
  steam_id bigint,
  demo_id bigint,
  PRIMARY KEY(steam_id, demo_id)
)
