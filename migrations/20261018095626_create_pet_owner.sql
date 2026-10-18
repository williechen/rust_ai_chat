-- Add migration script here
CREATE TABLE pet_owner (
    id TEXT PRIMARY KEY, -- PKNO
    name VARCHAR(255) NOT NULL, -- 主人名稱
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);