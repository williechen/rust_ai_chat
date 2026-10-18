-- Add migration script here
CREATE TABLE pet_profile (
    id TEXT PRIMARY KEY, -- PKNO
    name VARCHAR(255) NOT NULL, -- 寵物名稱
    ai_api_key VARCHAR(255), -- AI API 金鑰
    species VARCHAR(255) NOT NULL, -- 物種
    age INT, -- 年齡
    is_default_closure BOOLEAN DEFAULT FALSE, -- 是否預設關籠 
    owner_id TEXT REFERENCES pet_owner(id), -- 主人ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE pet_extra (
    id TEXT PRIMARY KEY, -- PKNO
    pet_id TEXT REFERENCES pet_profile(id), -- 寵物ID
    owner_id TEXT REFERENCES pet_owner(id), -- 主人ID
    category VARCHAR(255), -- 類別(例如: 興趣、個性、喜好、其他)
    content VARCHAR(255), -- 內容
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE pet_activity (
    id TEXT PRIMARY KEY, -- PKNO
    pet_id TEXT REFERENCES pet_profile(id), -- 寵物ID
    owner_id TEXT REFERENCES pet_owner(id), -- 主人ID
    activity_type VARCHAR(255), -- 活動類型(例如: 散步、玩耍、訓練)
    is_activity BOOLEAN DEFAULT FALSE, -- 是否進行中
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE pet_study(
    id TEXT PRIMARY KEY, -- PKNO
    pet_id TEXT REFERENCES pet_profile(id), -- 寵物ID
    owner_id TEXT REFERENCES pet_owner(id), -- 主人ID
    study_item VARCHAR(255), -- 學習項目(例如: 尋找、刪除、整理)
    is_study BOOLEAN DEFAULT FALSE, -- 是否進行中
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE pet_life_log (
    id TEXT PRIMARY KEY, -- PKNO
    pet_id TEXT REFERENCES pet_profile(id), -- 寵物ID
    owner_id TEXT REFERENCES pet_owner(id), -- 主人ID
    do_item VARCHAR(255), -- 執行項目(例如: 散步、玩耍、訓練)
    do_title TEXT, -- 標題(例如: 今天有散步，找到球)
    content TEXT, -- 內容(例如: 今天的散步情況．找到球的過程)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE codemap (
    id TEXT PRIMARY KEY, -- PKNO
    category VARCHAR(255), -- 類別(例如: 活動類型, 寵物屬性)
    code VARCHAR(255), -- 代碼(例如: WALK, PLAY, TRAIN)
    value VARCHAR(255), -- 值(例如: 散步、玩耍、訓練)
    description VARCHAR(255), -- 描述
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);