-- Add migration script here
CREATE TABLE product_events (
    source_app VARCHAR(32) NOT NULL
        CHECK (source_app IN ('web_chat', 'desktop_pet')),
    event_id VARCHAR(40) NOT NULL
        CHECK (char_length(event_id) = 36),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    event_type VARCHAR(128) NOT NULL
        CHECK (event_type <> '' AND event_type = btrim(event_type)),
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (source_app, event_id)
);