-- Your SQL goes here;
CREATE TYPE TX_KIND AS ENUM ('wrapper', 'protocol', 'transparent_transfer', 'shielded_transfer', 'bond', 'redelegation', 'unbond', 'withdraw', 'claim_rewards', 'reactivate_validator', 'deactivate_validator', 'ibc_envelop', 'ibc_transparent_transfer', 'ibc_shielded_transfer', 'change_consensus_key', 'change_commission', 'change_metadata', 'become_validator', 'init_account', 'init_proposal', 'resign_steward', 'reveal_public_key', 'unjail_validator', 'update_account', 'update_steward_commissions', 'proposal_vote','unknown');

CREATE TYPE TX_EXIT_STATUS AS ENUM ('applied', 'accepted', 'rejected');

CREATE TABLE transactions (
  id VARCHAR(64) PRIMARY KEY,
  inner_hash VARCHAR(64),
  index INT NOT NULL,
  kind TX_KIND NOT NULL,
  associated_data BYTEA,
  exit_status TX_EXIT_STATUS NOT NULL,
  gas_used INT NOT NULL,
  memo BYTEA,
  block_id VARCHAR(64) NOT NULL,
  CONSTRAINT fk_block FOREIGN KEY(block_id) REFERENCES blocks(id) ON DELETE CASCADE
);
