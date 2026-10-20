-- Add migration script here
CREATE TABLE mbr_session (
    id TEXT NOT NULL PRIMARY KEY,
    mbr_user_id TEXT NOT NULL REFERENCES mbr_user(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX idx_mbr_session_user_expires
    ON mbr_session(mbr_user_id, expires_at);