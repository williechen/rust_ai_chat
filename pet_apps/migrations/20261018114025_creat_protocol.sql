-- Add migration script here
CREATE TABLE pet_token_log(
    id TEXT NOT NULL PRIMARY KEY,
    token_src TEXT NOT NULL, -- 憑證來源(pet)
    token_type TEXT NOT NULL,  -- 憑證類型(jwt, sha256, md5)
    token_context TEXT NOT NULL,  -- 憑證內容
    
    ref_token_src TEXT,  -- 參考的憑證來源(analytic)
    ref_token_type TEXT,  -- 參考的憑證類型(jwt, sha256, md5)
    ref_token_context TEXT,  -- 參考的憑證內容

    expired_at TIMESTAMP,  -- 憑證過期時間
    token_status TEXT,  -- 憑證狀態
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pet_send_log(
    id TEXT NOT NULL PRIMARY KEY,
    local_token TEXT NOT NULL,  -- 本地的憑證
    remote_token TEXT,  -- 遠端的憑證
    json_data TEXT NOT NULL,  -- 本地的JSON數據
    send_status TEXT,  -- 發送狀態
    send_at TIMESTAMP,  -- 發送時間
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

