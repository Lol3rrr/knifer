-- Your SQL goes here
CREATE TABLE IF NOT EXISTS sessions (
  id bigint[2] PRIMARY KEY,
  data jsonb,
  expiry_date TEXT
)
