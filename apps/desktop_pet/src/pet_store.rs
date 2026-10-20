#[derive(Clone)]
pub struct PetStore {
    pub pool: sqlx::SqlitePool,
}

impl PetStore {
    pub async fn create(&self, input: &str) -> Result<String, String> {
        let name = crate::normalize_new_pet_name(input)?;
        let id = uuid::Uuid::new_v4().to_string();

        let mut tx = self.pool.begin().await.map_err(|_| "無法開始儲存")?;

        sqlx::query(
            r#"INSERT INTO pet_owner(id
                ,name
            ) VALUES ('local-owner','本機使用者') 
            ON CONFLICT(id) DO NOTHING"#,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| "無法儲存主人")?;

        sqlx::query(
            r#"INSERT INTO pet_profile(id
                ,name
                ,species
                ,owner_id
            ) VALUES (?1,?2,'tortoise','local-owner')"#,
        )
        .bind(&id)
        .bind(&name)
        .execute(&mut *tx)
        .await
        .map_err(|_| "無法儲存寵物")?;

        tx.commit().await.map_err(|_| "寵物提交失敗")?;

        let (saved_id, saved_name): (String, String) = sqlx::query_as(
            r#"SELECT id
                ,name 
            FROM pet_profile 
            WHERE id=?1"#,
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| "已提交但讀回失敗，請先查核資料，勿立即重送")?;

        Ok(format!("{saved_name}（{saved_id}）"))
    }
}

#[cfg(test)]
mod day22_tests {
    use super::*;
    #[sqlx::test(migrations = "./migrations")]
    async fn day22_create_pet_persists(pool: sqlx::SqlitePool) {
        let store = PetStore { pool: pool.clone() };
        assert!(store.create(" 小橘 ").await.unwrap().contains("小橘"));

        let row: (String, String, String) =
            sqlx::query_as(r#"SELECT name,species,owner_id FROM pet_profile"#)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            row,
            ("小橘".into(), "tortoise".into(), "local-owner".into())
        );
        assert!(store.create("  ").await.is_err());

        let (count,): (i64,) = sqlx::query_as(r#"SELECT count(*) FROM pet_profile"#)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        let (violations,): (i64,) =
            sqlx::query_as(r#"SELECT count(*) FROM pragma_foreign_key_check"#)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(violations, 0);
    }
}
