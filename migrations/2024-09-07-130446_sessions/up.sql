-- Your SQL goes here
CREATE TABLE IF NOT EXISTS sessions (
  id bigint[2] PRIMARY KEY,
  steamid TEXT,
  expiry_date TEXT
)
