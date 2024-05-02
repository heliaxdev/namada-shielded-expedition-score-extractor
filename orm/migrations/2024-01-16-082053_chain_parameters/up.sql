-- Your SQL goes here;
CREATE TABLE chain_parameters (
  id INT PRIMARY KEY, -- this is the epoch
  total_native_token_supply VARCHAR NOT NULL,
  total_staked_native_token VARCHAR NOT NULL,
  max_validators INT NOT NULL,
  pos_inflation VARCHAR NOT NULL,
  pgf_steward_inflation VARCHAR NOT NULL,
  pgf_treasury_inflation VARCHAR NOT NULL,
  pgf_treasury VARCHAR NOT NULL
);