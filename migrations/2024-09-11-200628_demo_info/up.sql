-- Your SQL goes here
CREATE TABLE IF NOT EXISTS processing_status (
  demo_id bigint PRIMARY KEY REFERENCES demos(demo_id),
  info int2 NOT NULL -- the processing_status of the basic demo info
);

CREATE TABLE IF NOT EXISTS demo_info (
  demo_id bigint PRIMARY KEY REFERENCES demos(demo_id),
  map TEXT NOT NULL
)
