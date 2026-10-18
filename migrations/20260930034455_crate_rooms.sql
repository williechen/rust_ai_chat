-- Add migration script here
CREATE TABLE rooms (
    id varchar(40) PRIMARY KEY,  -- 房間Pkno
    category_room_id varchar(40) NOT NULL, -- 聊天室種類Pkno
    name TEXT NOT NULL, -- 房間名稱
    created_at TIMESTAMP NOT NULL , -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

