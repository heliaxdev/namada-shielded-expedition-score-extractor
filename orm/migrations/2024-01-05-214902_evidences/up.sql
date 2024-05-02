-- Your SQL goes here
CREATE TYPE EVIDENCE_KIND AS ENUM ('duplicate_vote', 'light_client_attack');

CREATE TABLE evidences (
  id SERIAL PRIMARY KEY,
  kind EVIDENCE_KIND NOT NULL,
  validator_address VARCHAR NOT NULL,
  block_id VARCHAR(64) NOT NULL,
  CONSTRAINT fk_block FOREIGN KEY(block_id) REFERENCES blocks(id) ON DELETE CASCADE
);

