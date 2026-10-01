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
        .map_err(|error| RoomRepositoryError::Repository(error.to_string()))
    }
}
