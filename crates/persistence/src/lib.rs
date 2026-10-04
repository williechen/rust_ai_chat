use async_trait::async_trait;
use chat_application::RoomRepository;
use chat_application::RoomRepositoryError;
use chat_domain::Room;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}

#[derive(Debug, FromRow)]
pub struct RoomRow {
    pub id: String,
    pub category_room_id: String,
    pub name: String,
}

impl From<RoomRow> for Room {
    fn from(row: RoomRow) -> Self {
        Self {
            id: row.id,
            category_room_id: row.category_room_id,
            name: row.name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresRoomRepository {
    pool: PgPool,
}

impl PostgresRoomRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoomRepository for PostgresRoomRepository {
    async fn find_by_id(&self, room_id: &str) -> Result<Option<Room>, RoomRepositoryError> {
        sqlx::query_as::<_, RoomRow>(
            r#"SELECT id
                    , category_room_id
                    , name
                FROM rooms
                WHERE id = $1
            "#,
        )
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Room::from))
        .map_err(|error| RoomRepositoryError::Repository(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_repository_find_by_id() {
        dotenv::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        migrate(&db).await.expect("Failed to run migrations");

        sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind("room-rust")
            .execute(&db)
            .await
            .expect("Failed to clean existing test room");

        sqlx::query(
            r#"
            INSERT INTO rooms (
                id,
                category_room_id,
                name,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
        )
        .bind("room-rust")
        .bind("category-programming")
        .bind("Rust")
        .execute(&db)
        .await
        .expect("Failed to seed test room");

        let repo = PostgresRoomRepository::new(db.clone());

        let room = repo
            .find_by_id("room-rust")
            .await
            .expect("room query should succeed")
            .expect("room-rust should exist");

        assert_eq!(room.id, "room-rust");
        assert_eq!(room.category_room_id, "category-programming");
        assert_eq!(room.name, "Rust");

        sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind("room-rust")
            .execute(&db)
            .await
            .expect("Failed to clean test room");
    }
}
