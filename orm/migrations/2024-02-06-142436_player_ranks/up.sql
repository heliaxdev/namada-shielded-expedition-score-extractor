-- Your SQL goes here

CREATE TABLE player_ranks (
    id SERIAL PRIMARY KEY,
    ranking INT NOT NULL,
    player_id VARCHAR NOT NULL,
    CONSTRAINT fk_player FOREIGN KEY(player_id) REFERENCES players(id) ON DELETE CASCADE
);
