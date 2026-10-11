use server::ws_policy::{room_matches_connection, valid_message_content};

#[test]
fn matching_room() {
    assert!(room_matches_connection("room-a", "room-a"));
}
#[test]
fn different_room() {
    assert!(!room_matches_connection("room-a", "room-b"));
}
#[test]
fn blank_room() {
    assert!(!room_matches_connection("", ""));
    assert!(!room_matches_connection("room-a", " "));
}
#[test]
fn accepts_nonblank_content() {
    assert!(valid_message_content("你好"));
    assert!(valid_message_content(" 你好 "));
}
#[test]
fn rejects_blank_content() {
    assert!(!valid_message_content(""));
    assert!(!valid_message_content(" \n\t "));
}
