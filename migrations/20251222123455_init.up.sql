-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id NUMERIC PRIMARY KEY,
    role INTEGER
);

CREATE UNIQUE INDEX IF NOT EXISTS user_idx ON users (id);

CREATE TABLE IF NOT EXISTS match_history (
    id NUMERIC PRIMARY KEY,
    winner NUMERIC REFERENCES users(id) ON UPDATE CASCADE ON DELETE SET NULL,
    result INTEGER CHECK (result >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS match_history_idx ON match_history (id);
