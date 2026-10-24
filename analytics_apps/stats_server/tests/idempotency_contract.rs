use stats_server::{EventIdentity, InMemoryDeduplicator};
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

#[test]
fn event_identity_deduplicates_per_source() {
    let mut store = InMemoryDeduplicator::default();

    // 相同來源、相同 event_id 的重送，只接受第一筆。
    assert!(store.accept_once(EventIdentity::new("web_chat", "id-1")));
    assert!(!store.accept_once(EventIdentity::new("web_chat", "id-1")));

    // 相同 event_id 在不同來源之間彼此獨立。
    assert!(store.accept_once(EventIdentity::new("desktop_pet", "id-1")));

    // 同來源的新 event_id 仍應正常接受。
    assert!(store.accept_once(EventIdentity::new("web_chat", "id-2")));
}
