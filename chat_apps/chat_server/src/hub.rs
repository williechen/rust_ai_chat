use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use shared::ServerEvent;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct RoomHub {
    rooms: Arc<Mutex<HashMap<String, broadcast::Sender<ServerEvent>>>>,
    capacity: usize,
}

impl RoomHub {
    pub fn new(capacity: usize) -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    pub fn subscribe(
        &self,
        room_id: String,
    ) -> (
        broadcast::Sender<ServerEvent>,
        broadcast::Receiver<ServerEvent>,
    ) {
        let mut rooms = self.rooms.lock().unwrap();

        let sender = rooms
            .entry(room_id)
            .or_insert_with(|| {
                let (sender, _receiver) = broadcast::channel(self.capacity);
                sender
            })
            .clone();

        let receiver = sender.subscribe();

        (sender, receiver)
    }

    pub fn subscriber_count(&self, room_id: &str) -> usize {
        self.rooms
            .lock()
            .unwrap()
            .get(room_id)
            .map_or(0, |sender| sender.receiver_count())
    }

    pub fn publish(&self, room_id: String, event: ServerEvent) {
        let sender = {
            let rooms = self.rooms.lock().unwrap();
            rooms.get(&room_id).cloned()
        };
        if let Some(sender) = sender {
            let _ = sender.send(event);
        }
    }

    pub fn cleanup(&self, room_id: String, sender: &broadcast::Sender<ServerEvent>) {
        let mut rooms = self.rooms.lock().unwrap();
        let should_remove = rooms.get(&room_id).is_some_and(|current_sender| {
            current_sender.same_channel(sender) && current_sender.receiver_count() == 0
        });

        if should_remove {
            rooms.remove(&room_id);
        }
    }
}
