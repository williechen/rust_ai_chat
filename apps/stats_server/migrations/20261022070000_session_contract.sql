CREATE TABLE IF NOT EXISTS uac_login_session (
    token_hash TEXT PRIMARY KEY,
    user_id VARCHAR(40) NOT NULL REFERENCES uac_user(id) ON DELETE CASCADE,
    credential_fingerprint TEXT NOT NULL,
    restricted BOOLEAN NOT NULL,
    expires_at BIGINT NOT NULL,
    CONSTRAINT session_hash_length CHECK (char_length(token_hash) = 64),
    CONSTRAINT session_fingerprint_length CHECK (char_length(credential_fingerprint) = 64)
);
CREATE INDEX IF NOT EXISTS idx_uac_login_session_user ON uac_login_session(user_id);
CREATE INDEX IF NOT EXISTS idx_uac_login_session_expiry ON uac_login_session(expires_at);