use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    source: String,
    id: String,
}
trait EventStore {
    fn insert_once(&mut self, key: Key) -> bool;
}
#[derive(Default)]
struct FakeStore {
    seen: HashSet<Key>,
}
impl EventStore for FakeStore {
    fn insert_once(&mut self, key: Key) -> bool {
        self.seen.insert(key)
    }
}
#[test]
fn duplicate_is_counted_once_across_retries() {
    let mut store = FakeStore::default();
    let key = Key {
        source: "web_chat".into(),
        id: "event-1".into(),
    };
    assert!(store.insert_once(key.clone()));
    assert!(!store.insert_once(key));
    assert!(store.insert_once(Key {
        source: "desktop_pet".into(),
        id: "event-1".into()
    }));
}
