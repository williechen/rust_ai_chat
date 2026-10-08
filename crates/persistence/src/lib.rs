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

    async fn create(&self, room: &Room) -> Result<(), RoomRepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO rooms (
                id
                , category_room_id
                , name
                , created_at
                , updated_at
            )
            VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&room.id)
        .bind(&room.category_room_id)
        .bind(&room.name)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| RoomRepositoryError::Repository(error.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl chat_application::MessageRepository for PostgresMessageRepository {
    async fn create(
        &self,
        message: &chat_domain::ChatMessage,
    ) -> Result<(), chat_application::MessageRepositoryError> {
        // JSON 保留 MessageAuthor 各種類型與未來擴充欄位。
        let author = serde_json::to_value(&message.author)
            .map_err(|error| chat_application::MessageRepositoryError::Repository(error.to_string()))?;
        sqlx::query(
            "INSERT INTO messages (id, room_id, author, content) VALUES ($1, $2, $3, $4)",
        )
        .bind(&message.id)
        .bind(&message.room_id)
        .bind(author)
        .bind(&message.content)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| chat_application::MessageRepositoryError::Repository(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_room_repository_find_by_id(pool: PgPool) {
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
        .execute(&pool)
        .await
        .expect("Failed to seed test room");

        let repo = PostgresRoomRepository::new(pool);

        let room = repo
            .find_by_id("room-rust")
            .await
            .expect("room query should succeed")
            .expect("room-rust should exist");

        assert_eq!(room.id, "room-rust");
        assert_eq!(room.category_room_id, "category-programming");
        assert_eq!(room.name, "Rust");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_room_repository_create_then_find(pool: PgPool) {
        let repo = PostgresRoomRepository::new(pool);

        let room = Room {
            id: "room-create-test".to_string(),
            category_room_id: "default-category".to_string(),
            name: "Created Room".to_string(),
        };

        repo.create(&room)
            .await
            .expect("room create should succeed");

        let saved = repo
            .find_by_id(&room.id)
            .await
            .expect("room lookup should succeed")
            .expect("created room should exist");

        assert_eq!(saved, room);
    }
}
