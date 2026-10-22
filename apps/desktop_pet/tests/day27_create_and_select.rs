// apps/desktop_pet/tests/day27_create_and_select.rs
use desktop_pet::pet_store::PetStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::test]
async fn day27_new_pet_has_stable_identity_and_restart_selection() {
    let db_path = std::env::temp_dir().join(format!("pet27-{}.sqlite", uuid::Uuid::new_v4()));
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone())
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let store = PetStore { pool: pool.clone() };
    let created = store.create(" 阿龜 ").await.unwrap();
    assert_eq!(created.name, "阿龜");
    assert_eq!(created.species, "tortoise");
    assert!(uuid::Uuid::parse_str(&created.id).is_ok());
    assert_eq!(store.profile(&created.id).await.unwrap(), created);
    assert_eq!(store.select(&created.id).await.unwrap(), created);
    assert_eq!(store.active().await.unwrap(), Some(created.clone()));
    pool.close().await;
    let reopened = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    let reloaded = PetStore {
        pool: reopened.clone(),
    };
    assert_eq!(reloaded.active().await.unwrap(), Some(created));
    reopened.close().await;
    std::fs::remove_file(db_path).unwrap();
}
