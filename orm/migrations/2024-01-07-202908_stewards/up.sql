-- Your SQL goes here
CREATE TABLE stewards (
    id SERIAL PRIMARY KEY,
    namada_address VARCHAR NOT NULL,
    block_height INT NOT NULL
);

ALTER TABLE stewards
ADD UNIQUE (namada_address, block_height);