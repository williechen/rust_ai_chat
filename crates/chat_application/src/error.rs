use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoomRepositoryError {
    #[error("repository error: {0}")]
    Repository(String),
}

#[derive(Debug, thiserror::Error)]
pub enum MessageRepositoryError {
    #[error("message repository failure: {0}")]
    Repository(String),
}
