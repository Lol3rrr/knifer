-- Your SQL goes here
CREATE TABLE IF NOT EXISTS demos (
  steam_id TEXT NOT NULL,
  demo_id TEXT NOT NULL,
  uploaded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (steam_id, demo_id)
);
