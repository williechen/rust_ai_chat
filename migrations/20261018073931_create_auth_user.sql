-- Add migration script here
CREATE TABLE uac_user (
    id varchar(40) NOT NULL PRIMARY KEY, -- 使用者Pkno
    user_acnt TEXT NOT NULL, -- 使用者名稱
    user_pd TEXT NOT NULL, -- 密碼雜湊
    user_email TEXT NOT NULL, -- 電子郵件
    user_name TEXT NOT NULL, -- 使用者名稱
    user_status TEXT NOT NULL, -- 使用者狀態
    user_sexy TEXT NOT NULL, -- 使用者性別
    is_init_pd BOOLEAN NOT NULL, -- 是否初始化密碼
    expired_pd_at TIMESTAMP, -- 密碼過期時間
    last_login_at TIMESTAMP, -- 最後登入時間
    created_at TIMESTAMP NOT NULL, -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

CREATE TABLE uac_role (
    id varchar(40) NOT NULL PRIMARY KEY, -- 角色Pkno
    role_name TEXT NOT NULL, -- 角色名稱
    role_status TEXT NOT NULL, -- 角色狀態
    created_at TIMESTAMP NOT NULL, -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

CREATE TABLE uac_user_role (
    user_id varchar(40) NOT NULL, -- 使用者Pkno
    role_id varchar(40) NOT NULL, -- 角色Pkno
    created_at TIMESTAMP NOT NULL, -- 創建時間
    CONSTRAINT pk_uac_user_role PRIMARY KEY (user_id, role_id)
);

CREATE TABLE uac_acl (
    id varchar(40) NOT NULL PRIMARY KEY, -- 頁面Pkno
    acl_name TEXT NOT NULL, -- 頁面名稱
    acl_path TEXT NOT NULL, -- 頁面路徑
    acl_category TEXT NOT NULL, -- 頁面分類(項目: category，頁面: page，功能:action)
    acl_parent_id varchar(40), -- 父頁面Pkno
    acl_status TEXT NOT NULL, -- 頁面狀態
    created_at TIMESTAMP NOT NULL, -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

CREATE TABLE uac_role_acl (
    role_id varchar(40) NOT NULL, -- 角色Pkno
    acl_id varchar(40) NOT NULL, -- 頁面Pkno
    created_at TIMESTAMP NOT NULL, -- 創建時間
    CONSTRAINT pk_uac_role_acl PRIMARY KEY (role_id, acl_id)
);

CREATE TABLE uac_user_acl (
    user_id varchar(40) NOT NULL, -- 使用者Pkno
    acl_id varchar(40) NOT NULL, -- 頁面Pkno
    acl_filter TEXT NOT NULL, -- 頁面加減
    created_at TIMESTAMP NOT NULL, -- 創建時間
    CONSTRAINT pk_uac_user_acl PRIMARY KEY (user_id, acl_id)
);


CREATE TABLE mbr_user (
    id varchar(40) NOT NULL PRIMARY KEY, -- 使用者Pkno
    user_acnt TEXT NOT NULL, -- 使用者名稱
    user_pd TEXT NOT NULL, -- 密碼雜湊
    user_email TEXT NOT NULL, -- 電子郵件
    user_name TEXT NOT NULL, -- 使用者名稱
    user_status TEXT NOT NULL, -- 使用者狀態
    user_sexy TEXT NOT NULL, -- 使用者性別
    user_birthday TIMESTAMP NOT NULL, -- 使用者生日
    is_init_pd BOOLEAN NOT NULL, -- 是否初始化密碼
    expired_pd_at TIMESTAMP, -- 密碼過期時間
    last_login_at TIMESTAMP, -- 最後登入時間
    created_at TIMESTAMP NOT NULL, -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

CREATE TABLE mbr_role (
    id varchar(40) NOT NULL PRIMARY KEY, -- 角色Pkno
    role_name TEXT NOT NULL, -- 角色名稱
    role_status TEXT NOT NULL, -- 角色狀態
    created_at TIMESTAMP NOT NULL, -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

CREATE TABLE mbr_user_role (
    user_id varchar(40) NOT NULL, -- 使用者Pkno
    role_id varchar(40) NOT NULL, -- 角色Pkno
    created_at TIMESTAMP NOT NULL, -- 創建時間
    CONSTRAINT pk_mbr_user_role PRIMARY KEY (user_id, role_id)
);

CREATE TABLE mbr_perm (
    id varchar(40) NOT NULL PRIMARY KEY, -- 權限Pkno
    perm_name TEXT NOT NULL, -- 權限名稱
    perm_status TEXT NOT NULL, -- 權限狀態
    created_at TIMESTAMP NOT NULL, -- 創建時間
    updated_at TIMESTAMP NOT NULL -- 更新時間
);

CREATE TABLE mbr_role_perm (
    role_id varchar(40) NOT NULL, -- 角色Pkno
    perm_id varchar(40) NOT NULL, -- 權限Pkno
    created_at TIMESTAMP NOT NULL, -- 創建時間
    CONSTRAINT pk_mbr_role_perm PRIMARY KEY (role_id, perm_id)
);
    
CREATE TABLE mbr_user_perm (
    user_id varchar(40) NOT NULL, -- 使用者Pkno
    perm_id varchar(40) NOT NULL, -- 權限Pkno
    perm_filter TEXT NOT NULL, -- 權限加減
    created_at TIMESTAMP NOT NULL, -- 創建時間
    CONSTRAINT pk_mbr_user_perm PRIMARY KEY (user_id, perm_id)
);