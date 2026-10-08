use crate::state::AppState;
use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use shared::{ClientEvent, ServerEvent};
use tokio::sync::broadcast;

pub async fn ws_handler(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    match state.room_repository.find_by_id(&room_id).await {
        Ok(Some(_room)) => ws.on_upgrade(move |socket| handle_socket(socket, state, room_id)),
        Ok(None) => Response::builder()
            .status(404)
            .body("Room not found".into())
            .unwrap(),
        Err(error) => {
            eprintln!("websocket room lookup error: {:?}", error);
            Response::builder()
                .status(500)
                .body("Internal server error".into())
                .unwrap()
        }
    }
}

async fn handle_socket(mut socket: WebSocket, state: AppState, room_id: String) {
    let (room_sender, mut room_events) = state.room_hub.subscribe(room_id.clone());

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(result) = incoming else {
                    break;
                };
                let message = match result {
                    Ok(message) => message,
                    Err(error) => {
                        eprintln!("websocket receive error: {error}");
                        break;
                    }
                };

                if let Message::Text(text) = message {
                    if let Err(error) = handle_client_event(&mut socket, &state, text.as_str(), room_id.clone()).await {
                        eprintln!("client message error: {error}");
                        break;
                    }
                }
            }
            event = room_events.recv() => {
                match event {
                    Ok(event) => {
                        if let Err(error) = send_event(&mut socket, event).await {
                            eprintln!("websocket send error: {error}");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("websocket receiver lagged skipped: {skipped} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        eprintln!("websocket receiver closed");
                        break;
                    }
                }
            }
        }
    }

    drop(room_events);

    state.room_hub.cleanup(room_id.clone(), &room_sender);
}

async fn handle_client_event(
    socket: &mut WebSocket,
    state: &AppState,
    text: &str,
    event_room_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event: ClientEvent = serde_json::from_str(text)?;

    match event {
        ClientEvent::Ping => {
            send_event(socket, ServerEvent::Pong).await?;
        }
        ClientEvent::SendMessage { room_id, content } => {
            if event_room_id != room_id {
                send_event(
                    socket,
                    ServerEvent::Error {
                        code: "room_mismatch".to_string(),
                        message: "訊息所屬聊天室與目前連線不一致".to_string(),
                    },
                )
                .await?;
                return Ok(());
            }

            let message = state.chat_service.send_message(room_id.clone(), content);
            // 先完成資料庫寫入，成功後才對房間廣播。
            if let Err(error) =
                chat_application::persist_message(state.message_repository.as_ref(), &message).await
            {
                eprintln!("訊息持久化失敗：{error}");
                send_event(socket, ServerEvent::Error {
                    code: "message_persistence_failed".to_string(),
                    message: "訊息儲存失敗，請稍後重試".to_string(),
                }).await?;
                return Ok(());
            }
            state
                .room_hub
                .publish(room_id.clone(), ServerEvent::MessageCreated(message));
        }
        ClientEvent::Typing { .. } => {
            // 後面再做 presence / typing。
        }
    }

    Ok(())
}

async fn send_event(
    socket: &mut WebSocket,
    event: ServerEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string(&event)?;
    socket.send(Message::Text(json.into())).await?;
    Ok(())
}
