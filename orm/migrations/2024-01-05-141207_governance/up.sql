-- Your SQL goes here;
CREATE TYPE GOVERNANCE_KIND AS ENUM ('pgf_steward', 'pgf_funding', 'default', 'default_with_wasm');
CREATE TYPE GOVERNANCE_RESULT AS ENUM ('passed', 'rejected', 'pending', 'unknown', 'voting_period');

CREATE TABLE governance_proposals (
  id INT PRIMARY KEY,
  content VARCHAR,
  kind GOVERNANCE_KIND NOT NULL,
  author VARCHAR NOT NULL,
  start_epoch INT NOT NULL,
  end_epoch INT NOT NULL,
  grace_epoch INT NOT NULL,
  result GOVERNANCE_RESULT NOT NULL DEFAULT 'pending',
  yay_votes VARCHAR NOT NULL DEFAULT '0',
  nay_votes VARCHAR NOT NULL DEFAULT '0',
  abstain_votes VARCHAR NOT NULL DEFAULT '0',
  transaction_id VARCHAR(64) NOT NULL,
  CONSTRAINT fk_transaction FOREIGN KEY(transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
);

CREATE INDEX governance_proposal_kind ON governance_proposals (kind);

