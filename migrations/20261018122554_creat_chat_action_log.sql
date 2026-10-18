-- Add migration script here
CREATE TABLE chat_action_log (
    id TEXT NOT NULL PRIMARY KEY,
    action_type VARCHAR(255) NOT NULL, -- (例: 登入, 登出, 發送訊息，刪除訊息，建立聊天室，加入聊天室，離開聊天室, 修改訊息)
    action_content TEXT, -- (例: 動作的內容，例如訊息的文字)
    ref_table VARCHAR(255), -- (例: chat, message, user)
    ref_id TEXT, -- (例: 對應的 chat, message, user 的 id)
    ref_column VARCHAR(255), -- (例: 對應的 chat, message, user 的欄位名稱)
    created_user_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);