use pet_domain::cleanup::{CleanupError, CleanupOperation};

#[test]
fn preview_cannot_be_counted_or_completed() {
    let mut op = CleanupOperation::new();
    assert!(!op.may_emit_success_event());
    assert_eq!(op.complete(true), Err(CleanupError::ConfirmationRequired));
}
#[test]
fn confirmed_success_counts_exactly_after_completion() {
    let mut op = CleanupOperation::new();
    op.confirm().unwrap();
    assert!(!op.may_emit_success_event());
    op.complete(true).unwrap();
    assert!(op.may_emit_success_event());
    assert_eq!(op.complete(true), Err(CleanupError::ConfirmationRequired));
}
#[test]
fn failed_operation_never_counts() {
    let mut op = CleanupOperation::new();
    op.confirm().unwrap();
    op.complete(false).unwrap();
    assert!(!op.may_emit_success_event());
}
