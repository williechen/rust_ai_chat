-- Add migration script here
CREATE TABLE rooms (
    id varchar(40) PRIMARY KEY,
    category_room_id varchar(40) NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);