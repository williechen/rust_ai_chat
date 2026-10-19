-- Add migration script here
CREATE TABLE ai_person (
    id TEXT NOT NULL PRIMARY KEY, -- PKNO
    name VARCHAR(255) NOT NULL, -- 名稱
    role VARCHAR(255), -- 角色(例如: 助手、導師)
    description TEXT, -- 描述
    ai_api_key VARCHAR(255), -- AI API 金鑰
    person_status VARCHAR(255), -- 人物狀態(例如: 活躍、休息)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE ai_person_extra (
    id TEXT NOT NULL PRIMARY KEY, -- PKNO
    ai_person_id TEXT NOT NULL REFERENCES ai_person(id), -- AI 人物ID
    category VARCHAR(255), -- 類別(例如: 興趣、個性、喜好、其他)
    content VARCHAR(255), -- 內容
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

CREATE TABLE ai_person_activity (
    id TEXT NOT NULL PRIMARY KEY, -- PKNO
    ai_person_id TEXT NOT NULL REFERENCES ai_person(id), -- AI 人物ID
    activity_type VARCHAR(255), -- 活動類型(例如: 對話、教學, 亂數進房)
    is_activity BOOLEAN DEFAULT FALSE, -- 是否進行中
    is_use_ai BOOLEAN DEFAULT FALSE, -- 是否使用 AI
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

-- 今天進到某個房間的生活紀錄
CREATE TABLE ai_person_life_log (
    id TEXT NOT NULL PRIMARY KEY, -- PKNO
    ai_person_id TEXT NOT NULL REFERENCES ai_person(id), -- AI 人物ID
    room_id TEXT NOT NULL, -- 房間ID
    do_item VARCHAR(255), -- 執行項目(例如: 對話、教學)
    do_title TEXT, -- 標題(例如: 今天有對話，教學內容)
    content TEXT, -- 內容(例如: 今天的對話情況，教學過程)
    has_use_ai BOOLEAN DEFAULT FALSE, -- 有使用 AI
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- 建立時間
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP -- 更新時間
);

