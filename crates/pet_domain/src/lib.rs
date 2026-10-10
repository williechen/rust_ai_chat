pub mod cleanup;
pub mod error;
pub mod profile;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Previewed,
    Confirmed,
    Succeeded,
    Failed,
}
pub fn count_as_cleanup(state: OperationState) -> bool {
    state == OperationState::Succeeded
}
