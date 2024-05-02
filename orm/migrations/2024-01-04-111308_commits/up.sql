-- Your SQL goes here;
CREATE TABLE commits (
  id SERIAL PRIMARY KEY,
  signature VARCHAR,
  address VARCHAR NOT NULL,
  block_id VARCHAR(64) NOT NULL,
  CONSTRAINT fk_block FOREIGN KEY(block_id) REFERENCES blocks(id) ON DELETE CASCADE
);