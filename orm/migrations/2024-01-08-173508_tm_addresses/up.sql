-- Your SQL goes here;
CREATE TABLE tm_addresses (
  id SERIAL PRIMARY KEY,
  tm_address VARCHAR NOT NULL,
  epoch INT NOT NULL,
  validator_namada_address VARCHAR NOT NULL
);

ALTER TABLE tm_addresses
ADD UNIQUE (validator_namada_address, epoch);

ALTER TABLE tm_addresses
ADD UNIQUE (tm_address, epoch);

CREATE INDEX tm_addresses_validator_namada_address ON tm_addresses (validator_namada_address DESC);