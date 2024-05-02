-- Your SQL goes here
CREATE TYPE VOTE_KIND AS ENUM ('nay', 'yay', 'abstain');

CREATE TABLE governance_votes (
  id SERIAL PRIMARY KEY,
  kind VOTE_KIND NOT NULL,
  voter_address VARCHAR NOT NULL,
  proposal_id INT NOT NULL,
  transaction_id VARCHAR NOT NULL,
  player_id VARCHAR NOT NULL,
  CONSTRAINT fk_proposal FOREIGN KEY(proposal_id) REFERENCES governance_proposals(id) ON DELETE CASCADE,
  CONSTRAINT fk_transaction FOREIGN KEY(transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
);

ALTER TABLE governance_votes
ADD UNIQUE (voter_address, proposal_id);

CREATE INDEX governance_votes_kind ON governance_votes (kind);
