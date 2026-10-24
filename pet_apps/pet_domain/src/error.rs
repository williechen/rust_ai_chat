use crate::{PetCommand, PetState};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PetError {
    #[error("cannot execute {command:?} while pet is {state:?}")]
    InvalidTransition {
        state: PetState,
        command: PetCommand,
    },
}
