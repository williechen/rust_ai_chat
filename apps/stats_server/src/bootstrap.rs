use argon2::{Argon2, PasswordHasher};

pub async fn initialize(pool: &sqlx::PgPool, password: String) -> Result<String, String> {
    if !(8..=128).contains(&password.len()) {
        return Err("密碼須為 8 到 128 bytes".into());
    }
    let hash = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|_| "密碼雜湊失敗")
    })
    .await
    .map_err(|_| "密碼工作失敗")??;

    let mut tx = pool.begin().await.map_err(|_| "無法開始初始化")?;
    // 同時啟動的初始化會序列化；不使用先查後寫而無鎖的競態。
    sqlx::query("LOCK TABLE uac_user IN EXCLUSIVE MODE")
        .execute(&mut *tx)
        .await
        .map_err(|_| "無法鎖定帳號表")?;

    let exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
            SELECT 1 
            FROM uac_user 
            WHERE lower(user_acnt)='admin')"#,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| "無法查核管理員")?;
    if exists {
        return Err("管理員已存在；不覆寫密碼".into());
    }

    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(r#"INSERT INTO uac_user (id
        ,user_acnt
        ,user_pd
        ,user_email
        ,user_name
        ,user_status
        ,is_init_pd
        ,created_at
        ,updated_at
    ) VALUES ($1,'admin',$2,'admin@localhost.invalid','管理員','active',TRUE,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)"#)
        .bind(&id)
        .bind(hash)
        .execute(&mut *tx).await.map_err(|_| "管理員寫入失敗")?;

    tx.commit().await.map_err(|_| "管理員提交失敗")?;

    Ok(id)
}

#[cfg(all(test, feature = "postgres-integration"))]
mod day22_tests {
    use super::*;
    use argon2::{PasswordHash, PasswordVerifier};
    #[sqlx::test(migrations = "./migrations")]
    async fn day22_bootstrap_serializes_and_preserves_hash(pool: sqlx::PgPool) {
        let password = "lesson-only-password".to_owned();
        let (a, b) = tokio::join!(
            initialize(&pool, password.clone()),
            initialize(&pool, password.clone())
        );
        assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
        assert!(initialize(&pool, "short".into()).await.is_err());
        let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM uac_user")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
        let (hash, must_change): (String, bool) =
            sqlx::query_as("SELECT user_pd,is_init_pd FROM uac_user WHERE user_acnt='admin'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(must_change);
        assert!(hash.starts_with("$argon2id$"));
        Argon2::default()
            .verify_password(password.as_bytes(), &PasswordHash::new(&hash).unwrap())
            .unwrap();
        assert!(
            initialize(&pool, "different-lesson-password".into())
                .await
                .is_err()
        );
        let (after,): (String,) =
            sqlx::query_as("SELECT user_pd FROM uac_user WHERE user_acnt='admin'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(after, hash);
    }
}
