use pet_domain::cleanup::CleanupOperation;
use pet_domain::cleanup_event::succeeded_event;
use uuid::Uuid;

fn operation_id() -> String {
    Uuid::from_u128(88).to_string()
}

#[test]
fn preview_is_not_a_success_event() {
    let op = CleanupOperation::new();
    assert_eq!(succeeded_event(&op, &operation_id()).unwrap(), None);
}

#[test]
fn confirmation_without_filesystem_completion_is_not_success() {
    let mut op = CleanupOperation::new();
    op.confirm().unwrap();
    assert_eq!(succeeded_event(&op, &operation_id()).unwrap(), None);
}

#[test]
fn failed_operation_is_not_counted() {
    let mut op = CleanupOperation::new();
    op.confirm().unwrap();
    op.complete(false).unwrap();
    assert_eq!(succeeded_event(&op, &operation_id()).unwrap(), None);
}

#[test]
fn succeeded_operation_produces_stable_minimal_event() {
    let mut op = CleanupOperation::new();
    op.confirm().unwrap();
    // 測試 Domain 行為；真實 Adapter 是否成功需要獨立 E2E。
    op.complete(true).unwrap();
    let a = succeeded_event(&op, &operation_id()).unwrap().unwrap();
    let b = succeeded_event(&op, &operation_id()).unwrap().unwrap();
    assert_eq!(a, b);
    assert_eq!(a.event_id.to_string(), operation_id());
    assert_eq!(a.schema_version, 1);
    assert_eq!(a.source_app, "desktop_pet");
    assert_eq!(a.event_type, "pet.cleanup.succeeded");
    assert!(!format!("{a:?}").contains("/home/"));
}

#[test]
fn invalid_operation_id_never_falls_back_to_new_random_id() {
    let mut op = CleanupOperation::new();
    op.confirm().unwrap();
    op.complete(true).unwrap();
    assert!(succeeded_event(&op, "bad-id").is_err());
}
