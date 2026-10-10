#[test]
fn same_source_and_id_is_duplicate() {
    let mut store = InMemoryDeduplicator::default();
    let key = EventIdentity::new("web_chat", "event-001");
    assert!(store.accept_once(key.clone()));
    assert!(!store.accept_once(key));
}

#[test]
fn different_source_same_id_is_independent() {
    let mut store = InMemoryDeduplicator::default();
    assert!(store.accept_once(EventIdentity::new("web_chat", "same")));
    assert!(store.accept_once(EventIdentity::new("desktop_pet", "same")));
}

#[test]
fn distinct_ids_are_independent() {
    let mut store = InMemoryDeduplicator::default();
    assert!(store.accept_once(EventIdentity::new("web_chat", "a")));
    assert!(store.accept_once(EventIdentity::new("web_chat", "b")));
}
