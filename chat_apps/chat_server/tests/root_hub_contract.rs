use server::hub::RoomHub;
use shared::ServerEvent;
use tokio::sync::broadcast::error::TryRecvError;

#[test]
fn message_stays_inside_room() {
    let hub = RoomHub::new(8);
    let (_a_sender, mut a) = hub.subscribe("room-a".into());
    let (_b_sender, mut b) = hub.subscribe("room-b".into());
    hub.publish("room-a".into(), ServerEvent::Pong);
    assert_eq!(a.try_recv().unwrap(), ServerEvent::Pong);
    assert!(matches!(b.try_recv(), Err(TryRecvError::Empty)));
}

#[test]
fn subscribers_share_events() {
    let hub = RoomHub::new(8);
    let (_sender1, mut one) = hub.subscribe("same".into());
    let (_sender2, mut two) = hub.subscribe("same".into());
    hub.publish("same".into(), ServerEvent::Pong);
    assert_eq!(one.try_recv().unwrap(), ServerEvent::Pong);
    assert_eq!(two.try_recv().unwrap(), ServerEvent::Pong);
}

#[test]
fn last_receiver_cleanup_allows_new_subscription() {
    let hub = RoomHub::new(8);
    let (sender, receiver) = hub.subscribe("room".into());
    drop(receiver);
    hub.cleanup("room".into(), &sender);
    let (_new_sender, mut new_receiver) = hub.subscribe("room".into());
    hub.publish("room".into(), ServerEvent::Pong);
    assert_eq!(new_receiver.try_recv().unwrap(), ServerEvent::Pong);
}

#[test]
fn count_tracks_receiver_lifecycle() {
    let hub = RoomHub::new(8);
    assert_eq!(hub.subscriber_count("r"), 0);
    let (sender, receiver) = hub.subscribe("r".into());
    assert_eq!(hub.subscriber_count("r"), 1);
    drop(receiver);
    hub.cleanup("r".into(), &sender);
    assert_eq!(hub.subscriber_count("r"), 0);
}
