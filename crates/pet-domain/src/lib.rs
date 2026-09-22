#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetState {
    Idle,
    Listening,
    Thinking,
    Speaking,
    Sleeping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PetEvent {
    UserClicked,
    UserMessage(String),
    ModelStarted,
    ModelFinished,
    SleepRequested,
}

impl PetState {
    pub fn on_event(self, event: &PetEvent) -> Self {
        match (self, event) {
            (_, PetEvent::UserMessage(_)) => Self::Listening,
            (_, PetEvent::ModelStarted) => Self::Thinking,
            (_, PetEvent::ModelFinished) => Self::Speaking,
            (_, PetEvent::SleepRequested) => Self::Sleeping,
            (Self::Sleeping, PetEvent::UserClicked) => Self::Idle,
            (state, _) => state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_moves_pet_to_listening() {
        let state = PetState::Idle;
        assert_eq!(
            state.on_event(&PetEvent::UserMessage("hello".into())),
            PetState::Listening
        );
    }
}
