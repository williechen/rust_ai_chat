use serde::{Deserialize, Serialize};

/// 可顯示的狀態；動畫與 UI 不屬於 domain。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetState {
    #[default]
    Idle,
    Interacting,
    Sleeping,
}

/// 外部送入狀態機的意圖，與輸出的事件分開。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub state: PetState,
    pub command: PetCommand,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot apply {:?} in {:?}", self.command, self.state)
    }
}

impl std::error::Error for InvalidTransition {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Default)]
pub struct PetMachine {
    snapshot: PetSnapshot,
}

impl PetMachine {
    pub fn snapshot(&self) -> PetSnapshot {
        self.snapshot
    }

    /// 無效命令不改變狀態；只有真正轉移才產生事件與遞增版本。
    pub fn dispatch(&mut self, command: PetCommand) -> Result<PetEvent, InvalidTransition> {
        let from = self.snapshot.state;
        let to = match (from, command) {
            (PetState::Idle, PetCommand::Interact) => PetState::Interacting,
            (PetState::Interacting, PetCommand::FinishInteraction) => PetState::Idle,
            (PetState::Idle | PetState::Interacting, PetCommand::Sleep) => PetState::Sleeping,
            (PetState::Sleeping, PetCommand::Wake) => PetState::Idle,
            _ => {
                return Err(InvalidTransition {
                    state: from,
                    command,
                });
            }
        };
        self.snapshot.state = to;
        self.snapshot.revision += 1;
        Ok(PetEvent::StateChanged { from, to })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_interact_finish_sleep_wake() {
        let mut pet = PetMachine::default();
        assert_eq!(
            pet.dispatch(PetCommand::Interact),
            Ok(PetEvent::StateChanged {
                from: PetState::Idle,
                to: PetState::Interacting
            })
        );
        assert_eq!(
            pet.dispatch(PetCommand::FinishInteraction),
            Ok(PetEvent::StateChanged {
                from: PetState::Interacting,
                to: PetState::Idle
            })
        );
        assert_eq!(
            pet.dispatch(PetCommand::Sleep),
            Ok(PetEvent::StateChanged {
                from: PetState::Idle,
                to: PetState::Sleeping
            })
        );
        assert_eq!(
            pet.dispatch(PetCommand::Wake),
            Ok(PetEvent::StateChanged {
                from: PetState::Sleeping,
                to: PetState::Idle
            })
        );
        assert_eq!(pet.snapshot().revision, 4);
    }

    #[test]
    fn invalid_transition_preserves_snapshot() {
        let mut pet = PetMachine::default();
        pet.dispatch(PetCommand::Sleep).unwrap();
        let before = pet.snapshot();
        assert_eq!(
            pet.dispatch(PetCommand::Interact),
            Err(InvalidTransition {
                state: PetState::Sleeping,
                command: PetCommand::Interact
            })
        );
        assert_eq!(pet.snapshot(), before);
    }

    #[test]
    fn interaction_can_be_interrupted_by_sleep() {
        let mut pet = PetMachine::default();
        pet.dispatch(PetCommand::Interact).unwrap();
        assert_eq!(
            pet.dispatch(PetCommand::Sleep),
            Ok(PetEvent::StateChanged {
                from: PetState::Interacting,
                to: PetState::Sleeping
            })
        );
    }
}
