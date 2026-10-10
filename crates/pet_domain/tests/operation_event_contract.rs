#[test]
fn preview_is_not_completed_cleanup() {
    assert!(!count_as_cleanup(OperationState::Previewed));
    assert!(!count_as_cleanup(OperationState::Confirmed));
    assert!(!count_as_cleanup(OperationState::Failed));
    assert!(count_as_cleanup(OperationState::Succeeded));
}
