-- Your SQL goes here
CREATE TABLE blocks (
    id VARCHAR(64) PRIMARY KEY,
    height INT NOT NULL UNIQUE CHECK(height > 0),
    proposer_address VARCHAR NOT NULL,
    included_at TIMESTAMP NOT NULL,
    epoch INT NOT NULL
);

CREATE INDEX height_asc ON blocks (height ASC);
CREATE INDEX height_desc ON blocks (height DESC);