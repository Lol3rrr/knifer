-- Your SQL goes here
CREATE TABLE IF NOT EXISTS demo_round (
  demo_id TEXT NOT NULL,
  round_number int2 NOT NULL,
  start_tick bigint NOT NULL,
  end_tick bigint NOT NULL,
  win_reason TEXT NOT NULL,
  events JSON NOT NULL,
  PRIMARY KEY (demo_id, round_number)
);
