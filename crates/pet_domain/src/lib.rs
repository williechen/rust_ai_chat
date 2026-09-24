pub mod error;

use crate::error::PetError;
use serde::{Deserialize, Serialize};

/// 可顯示的狀態；動畫與 UI 不屬於 domain。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetState {
    Idle,
    Interacting,
    Sleeping,
}

/// 外部送入狀態機的意圖，與輸出的事件分開。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetCommand {
    Interact,
    FinishInteraction,
    Sleep,
    Wake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PetEvent {
    StateChanged { from: PetState, to: PetState },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetSnapshot {
    pub state: PetState,
    pub revision: u64,
}

impl Default for PetSnapshot {
    fn default() -> Self {
        Self {
            state: PetState::Idle,
            revision: 0,
        }
    }
}

#[derive(Debug)]
pub struct PetMachine {
    state: PetState,
    revision: u64,
}

impl PetMachine {
    pub fn snapshot(&self) -> PetSnapshot {
        PetSnapshot {
            state: self.state,
            revision: self.revision,
        }
    }
    pub fn dispatch(&mut self, command: PetCommand) -> Result<PetSnapshot, PetError> {
        let next_state = match (self.state, command) {
            (PetState::Idle, PetCommand::Interact) => PetState::Interacting,
            (PetState::Interacting, PetCommand::FinishInteraction) => PetState::Idle,
            (PetState::Idle | PetState::Interacting, PetCommand::Sleep) => PetState::Sleeping,
            (PetState::Sleeping, PetCommand::Wake) => PetState::Idle,
            (state, command) => {
                return Err(PetError::InvalidTransition {
                    state,
                    command: command.clone(),
                });
            }
        };

        self.state = next_state;
        self.revision += 1;
        Ok(self.snapshot())
    }
}

impl Default for PetMachine {
    fn default() -> Self {
        Self {
            state: PetState::Idle,
            revision: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PetError;

    #[test]
    fn pet_starts_idle() {
        let pet = PetMachine::default();

        assert_eq!(
            pet.snapshot(),
            PetSnapshot {
                state: PetState::Idle,
                revision: 0,
            }
        );
    }

    #[test]
    fn interact_moves_idle_to_interacting() {
        let mut pet = PetMachine::default();

        let snapshot = pet.dispatch(PetCommand::Interact).unwrap();

        assert_eq!(snapshot.state, PetState::Interacting);
        assert_eq!(snapshot.revision, 1);
    }

    #[test]
    fn sleeping_pet_cannot_interact() {
        let mut pet = PetMachine::default();

        pet.dispatch(PetCommand::Sleep).unwrap();

        let before = pet.snapshot();

        let result = pet.dispatch(PetCommand::Interact);

        assert!(matches!(result, Err(PetError::InvalidTransition { .. })));

        assert_eq!(pet.snapshot(), before);
    }

    #[test]
    fn wake_moves_sleeping_pet_back_to_idle() {
        let mut pet = PetMachine::default();

        pet.dispatch(PetCommand::Sleep).unwrap();

        let snapshot = pet.dispatch(PetCommand::Wake).unwrap();

        assert_eq!(snapshot.state, PetState::Idle);
        assert_eq!(snapshot.revision, 2);
    }
}
