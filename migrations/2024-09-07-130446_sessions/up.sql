-- Your SQL goes here
CREATE TABLE IF NOT EXISTS sessions (
  id TEXT NOT NULL PRIMARY KEY,
  steamid TEXT,
  expiry_date TEXT NOT NULL
)
