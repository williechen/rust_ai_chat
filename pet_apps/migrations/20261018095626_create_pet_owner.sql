-- Add migration script here
CREATE TABLE pet_owner (
    id TEXT NOT NULL PRIMARY KEY, -- PKNO
    name VARCHAR(255) NOT NULL, -- 主人名稱
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE local_pet_selection (
 slot INTEGER PRIMARY KEY CHECK(slot=1),
 pet_id TEXT NOT NULL REFERENCES pet_profile(id) ON DELETE CASCADE
);