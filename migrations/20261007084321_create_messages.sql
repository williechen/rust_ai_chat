-- Add migration script here
CREATE TABLE messages (
    id VARCHAR(40) PRIMARY KEY,
    room_id VARCHAR(40) NOT NULL,
    author_kind VARCHAR(32) NOT NULL,
    author_id VARCHAR(255),
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_messages_room FOREIGN KEY (room_id) REFERENCES rooms(id),
    CONSTRAINT ck_messages_author_kind
        CHECK (author_kind IN ('anonymous_user', 'user', 'ai', 'system')),
    CONSTRAINT ck_messages_author_id
        CHECK (
            (author_kind IN ('anonymous_user', 'system') AND author_id IS NULL)
            OR
            (author_kind IN ('user', 'ai') AND author_id IS NOT NULL)
        )
);