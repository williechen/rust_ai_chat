use crate::cleanup::CleanupOperation;
use uuid::Uuid;

/// 只包含統計所需的最小事件欄位；不可攜帶私人檔案路徑。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupSucceededEvent {
    pub schema_version: u16,
    pub event_id: Uuid,
    pub source_app: &'static str,
    pub event_type: &'static str,
}

/// 此映射只能由可信任 filesystem Adapter 收到「確實成功」結果、
/// 並呼叫 operation.complete(true) 之後使用。
/// 僅 Domain 成功狀態不構成實際清理憑證。
pub fn succeeded_event(
    operation: &CleanupOperation,
    operation_id: &str,
) -> Result<Option<CleanupSucceededEvent>, uuid::Error> {
    if !operation.may_emit_success_event() {
        return Ok(None);
    }
    let event_id = Uuid::parse_str(operation_id)?;
    Ok(Some(CleanupSucceededEvent {
        schema_version: 1,
        event_id,
        source_app: "desktop_pet",
        event_type: "pet.cleanup.succeeded",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_operation_has_minimal_event() {
        let id = "50d3b2d1-512c-4c84-a1da-e8181a10cd11";
        let mut op = CleanupOperation::new();
        assert!(succeeded_event(&op, id).unwrap().is_none());
        op.confirm().unwrap();
        assert!(succeeded_event(&op, id).unwrap().is_none());
        op.complete(true).unwrap();
        let event = succeeded_event(&op, id).unwrap().unwrap();
        assert_eq!(event.schema_version, 1);
        assert_eq!(event.event_id, Uuid::parse_str(id).unwrap());
        assert_eq!(event.source_app, "desktop_pet");
        assert_eq!(event.event_type, "pet.cleanup.succeeded");
    }
    #[test]
    fn unsuccessful_operation_emits_nothing() {
        let mut op = CleanupOperation::new();
        op.confirm().unwrap();
        op.complete(false).unwrap();
        assert!(succeeded_event(&op, "bad-uuid").unwrap().is_none());
    }
}
