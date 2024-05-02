-- Your SQL goes here

CREATE TABLE unidentified_tasks (
    id SERIAL PRIMARY KEY,
    tx_kind TX_KIND NOT NULL,
    --completed_at INT NOT NULL,
    player_id VARCHAR NOT NULL,
    CONSTRAINT fk_player FOREIGN KEY(player_id) REFERENCES players(id) ON DELETE CASCADE
);

-- allows us to keep a single task completed per player
ALTER TABLE unidentified_tasks
ADD UNIQUE (player_id, tx_kind);
