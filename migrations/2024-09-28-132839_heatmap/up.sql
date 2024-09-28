-- Your SQL goes here
CREATE TABLE IF NOT EXISTS demo_heatmaps (
  demo_id bigint REFERENCES demo_info(demo_id),
  steam_id TEXT NOT NULL,
  data TEXT NOT NULL,
  PRIMARY KEY (demo_id, steam_id)
);
