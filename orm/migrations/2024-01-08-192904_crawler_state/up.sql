-- Your SQL goes here
CREATE TABLE crawler_state (
  id SERIAL PRIMARY KEY,
  height INT NOT NULL,
  epoch INT NOT NULL
);

ALTER TABLE crawler_state
ADD UNIQUE (height, epoch);