-- Add migration script here
CREATE TABLE uac_login_session (
    token_hash TEXT PRIMARY KEY,
    user_id VARCHAR(40) NOT NULL REFERENCES uac_user(id) ON DELETE CASCADE,
    credential_fingerprint TEXT NOT NULL,
    restricted BOOLEAN NOT NULL,
    expires_at BIGINT NOT NULL
);