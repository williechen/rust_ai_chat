use pet_domain::cleanup::{CleanupError, CleanupOperation};

#[test]
fn preview_cannot_be_counted_or_completed() {
    let mut operation = CleanupOperation::new();

    // 尚在預覽狀態不得記作清理成功。
    assert!(!operation.may_emit_success_event());
    // 沒有明確確認時，不能跳過確認直接標記完成。
    assert_eq!(
        operation.complete(true),
        Err(CleanupError::ConfirmationRequired)
    );
}

#[test]
fn confirmed_success_counts_only_after_completion() {
    let mut operation = CleanupOperation::new();
    operation.confirm().expect("預覽後應允許確認");

    // 即使已確認，只要未完成仍不能發送成功事件。
    assert!(!operation.may_emit_success_event());
    operation.complete(true).expect("已確認才能完成");
    assert!(operation.may_emit_success_event());

    // 已完成的操作不可再次完成或重複累計。
    assert_eq!(
        operation.complete(true),
        Err(CleanupError::ConfirmationRequired)
    );
}

#[test]
fn failed_operation_never_counts() {
    let mut operation = CleanupOperation::new();
    operation.confirm().expect("預覽後應允許確認");

    // 實際清理失敗時不得發送成功事件。
    operation.complete(false).expect("已確認後可記錄失敗");
    assert!(!operation.may_emit_success_event());
}
