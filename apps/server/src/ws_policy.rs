// 訊息內容不可僅有空白；保留原始內容，不自動裁切。
pub fn valid_message_content(content: &str) -> bool {
    !content.trim().is_empty()
}

// 防止跨房間發送；空白識別碼不可視為合法房間。
pub fn room_matches_connection(connection: &str, event: &str) -> bool {
    !connection.trim().is_empty() && !event.trim().is_empty() && connection == event
}
