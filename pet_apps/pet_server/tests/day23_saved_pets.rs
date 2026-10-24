use pet_server::pet_store::PetStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
#[tokio::test]
async fn day23_control_list_survives_pool_reopen() {
    let path = std::env::temp_dir().join(format!("day23-pet-{}.sqlite", uuid::Uuid::new_v4()));
    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options.clone())
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    let store = PetStore { pool: pool.clone() };
    assert!(store.list().await.unwrap().is_empty());
    store.create(" 小橘 ").await.unwrap();
    let before = store.list().await.unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].name, "小橘");
    assert_eq!(before[0].species, "tortoise");
    assert!(uuid::Uuid::parse_str(&before[0].id).is_ok());
    let encoded = serde_json::to_value(&before[0]).unwrap();
    assert_eq!(encoded.as_object().unwrap().len(), 3); // 不回傳主人或AI密鑰。
    assert!(store.create("  ").await.is_err());
    assert_eq!(store.list().await.unwrap(), before);
    pool.close().await;
    let reopened = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    let store = PetStore {
        pool: reopened.clone(),
    };
    assert_eq!(store.list().await.unwrap(), before);
    sqlx::migrate!("../migrations")
        .run(&reopened)
        .await
        .unwrap();
    assert_eq!(store.list().await.unwrap(), before);
    reopened.close().await;
    std::fs::remove_file(path).unwrap();
}
