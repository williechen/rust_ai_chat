-- Add migration script here
CREATE TABLE mbr_room (
    id TEXT PRIMARY KEY, --PKNO
    mbr_user_id VARCHAR(255) NOT NULL, --FK to mbr_user.id
    room_role VARCHAR(255) NOT NULL, -- 在這間房裡的角色(例如: 房主、協助者、一般成員)
    room_id VARCHAR(255) NOT NULL, --FK to room.id
    room_status VARCHAR(255) NOT NULL, -- 房間狀態(例如: 開放、封鎖、私人、隱藏)
    room_pd VARCHAR(255), -- 房間密碼
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE mbr_exclude (
    id TEXT PRIMARY KEY,
    mbr_user_id VARCHAR(255) NOT NULL, -- FK to mbr_user.id
    room_id VARCHAR(255) NOT NULL, -- FK to room.id
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);