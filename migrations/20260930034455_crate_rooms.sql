-- Add migration script here
CREATE TABLE rooms (
    id varchar(40) PRIMARY KEY,
    category_room_id varchar(40) NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL ,
    updated_at TIMESTAMP NOT NULL
);