#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupState {
    Previewed,
    Confirmed,
    Succeeded,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupError {
    ConfirmationRequired,
    AlreadyFinished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CleanupOperation {
    state: CleanupState,
}

impl CleanupOperation {
    pub fn new() -> Self {
        Self {
            state: CleanupState::Previewed,
        }
    }
    pub fn confirm(&mut self) -> Result<(), CleanupError> {
        if self.state != CleanupState::Previewed {
            return Err(CleanupError::AlreadyFinished);
        }
        self.state = CleanupState::Confirmed;
        Ok(())
    }
    pub fn complete(&mut self, actual_success: bool) -> Result<(), CleanupError> {
        if self.state != CleanupState::Confirmed {
            return Err(CleanupError::ConfirmationRequired);
        }
        self.state = if actual_success {
            CleanupState::Succeeded
        } else {
            CleanupState::Failed
        };
        Ok(())
    }
    pub fn may_emit_success_event(&self) -> bool {
        self.state == CleanupState::Succeeded
    }
}

impl Default for CleanupOperation {
    fn default() -> Self {
        Self::new()
    }
}
