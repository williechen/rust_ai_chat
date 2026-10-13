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
        match self.state {
            CleanupState::Previewed => Err(CleanupError::ConfirmationRequired),
            CleanupState::Succeeded | CleanupState::Failed => Err(CleanupError::AlreadyFinished),
            CleanupState::Confirmed => {
                self.state = if actual_success {
                    CleanupState::Succeeded
                } else {
                    CleanupState::Failed
                };
                Ok(())
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_is_one_time() {
        let mut op = CleanupOperation::new();
        assert_eq!(op.complete(true), Err(CleanupError::ConfirmationRequired));
        op.confirm().unwrap();
        op.complete(true).unwrap();
        assert!(op.may_emit_success_event());
        assert_eq!(op.complete(false), Err(CleanupError::AlreadyFinished));
        assert!(op.may_emit_success_event());
    }
    #[test]
    fn failure_never_emits_success() {
        let mut op = CleanupOperation::new();
        op.confirm().unwrap();
        op.complete(false).unwrap();
        assert!(!op.may_emit_success_event());
        assert_eq!(op.complete(true), Err(CleanupError::AlreadyFinished));
    }
}
