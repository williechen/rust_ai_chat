use pet_server::pet_store::PetStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[tokio::test]
async fn day24_explicit_selection_survives_reopen_and_rejects_unknown_id() {
    let path = std::env::temp_dir().join(format!("day24-pet-{}.sqlite", uuid::Uuid::new_v4()));

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
    assert!(store.active().await.unwrap().is_none());

    store.create("小橘").await.unwrap();
    store.create("小綠").await.unwrap();
    assert!(store.active().await.unwrap().is_none());

    let chosen = store.list().await.unwrap().remove(1);
    assert_eq!(store.select(&chosen.id).await.unwrap(), chosen);
    assert!(store.select("missing-id").await.is_err());
    assert_eq!(store.active().await.unwrap(), Some(chosen.clone()));

    pool.close().await;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::migrate!("../migrations").run(&pool).await.unwrap();
    assert_eq!(
        (PetStore { pool: pool.clone() }).active().await.unwrap(),
        Some(chosen)
    );
    pool.close().await;
    std::fs::remove_file(path).unwrap();
}
