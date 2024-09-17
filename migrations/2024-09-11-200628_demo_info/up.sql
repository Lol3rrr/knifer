-- Your SQL goes here
CREATE TABLE IF NOT EXISTS processing_status (
  demo_id bigint PRIMARY KEY REFERENCES demos(demo_id),
  info int2 NOT NULL -- the processing_status of the basic demo info
);

CREATE TABLE IF NOT EXISTS demo_info (
  demo_id bigint PRIMARY KEY REFERENCES demos(demo_id),
  map TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS demo_players (
  demo_id bigint REFERENCES demo_info(demo_id),
  steam_id TEXT NOT NULL,
  name TEXT NOT NULL,
  team int2 NOT NULL,
  color int2 NOT NULL,
  PRIMARY KEY (demo_id, steam_id)
);

CREATE TABLE IF NOT EXISTS demo_player_stats (
  demo_id bigint REFERENCES demo_info(demo_id),
  steam_id TEXT NOT NULL,
  kills int2 NOT NULL,
  deaths int2 NOT NULL,
  PRIMARY KEY (demo_id, steam_id)
);
