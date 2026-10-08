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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct PostgresMessageRepository {
    pool: sqlx::PgPool,
}

impl PostgresMessageRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl chat_application::MessageRepository for PostgresMessageRepository {
    async fn create(
        &self,
        message: &chat_domain::ChatMessage,
    ) -> Result<(), chat_application::MessageRepositoryError> {
        // 僅在 Infrastructure 映射 domain enum 到既有資料欄位。
        let (kind, author_id): (&str, Option<&str>) = match &message.author {
            chat_domain::MessageAuthor::AnonymousUser => ("anonymous_user", None),
            chat_domain::MessageAuthor::User { user_id } => ("user", Some(user_id)),
            chat_domain::MessageAuthor::Ai { persona_id } => ("ai", Some(persona_id)),
            chat_domain::MessageAuthor::System => ("system", None),
        };
        // 標準 INSERT 語意；$1~$5 僅是 PostgreSQL driver placeholder。
        sqlx::query(
            "INSERT INTO messages (
                id
                , room_id
                , author_kind
                , author_id
                , content)
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&message.id)
        .bind(&message.room_id)
        .bind(kind)
        .bind(author_id)
        .bind(&message.content)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| chat_application::MessageRepositoryError::Repository(e.to_string()))
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

    // 沿用既有 tests 模組的 use super::*，及根目錄的 migrations。
    // 每個 sqlx::test 取得獨立的測試資料庫，先套用 Migration。
    async fn insert_test_room(pool: &PgPool, id: &str) {
        sqlx::query(
            "INSERT INTO rooms (id, category_room_id, name, created_at, updated_at)
         VALUES ($1, $2, $3, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(id)
        .bind("category-programming")
        .bind("整合測試房間")
        .execute(pool)
        .await
        .expect("建立測試房間");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_message_repository_all_author_kinds(pool: PgPool) {
        use chat_application::MessageRepository;
        use chat_domain::{ChatMessage, MessageAuthor};

        insert_test_room(&pool, "message-room").await;
        let repo = PostgresMessageRepository::new(pool.clone());
        let inputs = [
            (
                "msg-anon",
                MessageAuthor::AnonymousUser,
                "anonymous_user",
                None,
            ),
            (
                "msg-user",
                MessageAuthor::User {
                    user_id: "u-1".into(),
                },
                "user",
                Some("u-1"),
            ),
            (
                "msg-ai",
                MessageAuthor::Ai {
                    persona_id: "p-1".into(),
                },
                "ai",
                Some("p-1"),
            ),
            ("msg-system", MessageAuthor::System, "system", None),
        ];
        for (id, author, expected_kind, expected_author_id) in inputs {
            let message = ChatMessage {
                id: id.to_string(),
                room_id: "message-room".to_string(),
                author,
                content: format!("內容-{id}"),
            };
            repo.create(&message).await.expect("寫入訊息");
            let row: (String, String, String, Option<String>, String) = sqlx::query_as(
                "SELECT id, room_id, author_kind, author_id, content
                 FROM messages WHERE id = $1",
            )
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("重新查詢訊息");
            assert_eq!(row.0, id);
            assert_eq!(row.1, "message-room");
            assert_eq!(row.2, expected_kind);
            assert_eq!(row.3.as_deref(), expected_author_id);
            assert_eq!(row.4, format!("內容-{id}"));
        }
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_message_repository_rejects_missing_room(pool: PgPool) {
        use chat_application::MessageRepository;
        let repo = PostgresMessageRepository::new(pool);
        let message = chat_domain::ChatMessage {
            id: "orphan-message".to_string(),
            room_id: "missing-room".to_string(),
            author: chat_domain::MessageAuthor::AnonymousUser,
            content: "不應成功".to_string(),
        };
        assert!(
            repo.create(&message).await.is_err(),
            "FK 應拒絕不存在的房間"
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_message_schema_rejects_invalid_author_and_length(pool: PgPool) {
        insert_test_room(&pool, "constraint-room").await;
        // 直接使用 SQL 測試 DB CHECK / 長度約束；Domain API 不會產生非法 enum。
        for (id, kind, author_id) in [
            ("bad-user", "user", None),
            ("bad-anon", "anonymous_user", Some("unexpected")),
            ("bad-kind", "unknown", None),
        ] {
            let result = sqlx::query(
                "INSERT INTO messages (id, room_id, author_kind, author_id, content)
             VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind("constraint-room")
            .bind(kind)
            .bind(author_id)
            .bind("測試內容")
            .execute(&pool)
            .await;
            assert!(result.is_err(), "{id} 應被資料庫約束拒絕");
        }
        let long_id = "x".repeat(41);
        let result = sqlx::query(
            "INSERT INTO messages (id, room_id, author_kind, content)
         VALUES ($1, $2, $3, $4)",
        )
        .bind(&long_id)
        .bind("constraint-room")
        .bind("system")
        .bind("不應成功")
        .execute(&pool)
        .await;
        assert!(result.is_err(), "id VARCHAR(40) 不應接受 41 字元");
    }
}
