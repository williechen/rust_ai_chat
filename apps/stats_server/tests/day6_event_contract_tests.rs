use stats_server::{EventIdentity, InMemoryDeduplicator};

#[test]
fn duplicate_same_source_is_rejected_but_other_source_is_allowed() {
    let mut store = InMemoryDeduplicator::default();
    let a = EventIdentity::new("web_chat", "evt-001");
    assert!(store.accept_once(a.clone()));
    assert!(!store.accept_once(a));
    assert!(store.accept_once(EventIdentity::new("desktop_pet", "evt-001")));
}
