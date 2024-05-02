-- Your SQL goes here
CREATE TABLE task_completion_state (
    id SERIAL PRIMARY KEY,
    last_processed_time TIMESTAMP NOT NULL,
    last_processed_height INT NOT NULL
);
