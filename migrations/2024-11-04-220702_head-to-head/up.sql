-- Your SQL goes here
CREATE TABLE IF NOT EXISTS demo_head_to_head (
  demo_id TEXT NOT NULL,
  player TEXT NOT NULL,
  enemy TEXT NOT NULL,
  kills int2 NOT NULL,
  PRIMARY KEY (demo_id, player, enemy)
);
