-- Add migration script here
-- 若既有帳號或 Email 大小寫衝突，migration 失敗並交由人工處理；不得刪資料。
CREATE UNIQUE INDEX uq_mbr_user_account ON mbr_user(lower(user_acnt));
CREATE UNIQUE INDEX uq_mbr_user_email ON mbr_user(lower(user_email));