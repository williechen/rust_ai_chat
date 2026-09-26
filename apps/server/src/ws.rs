use crate::state::AppState;
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use shared::{ClientEvent, ServerEvent};
use tokio::sync::broadcast;

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut events = state.events.subscribe();

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
                    if let Err(error) = handle_client_event(&mut socket, &state, text.as_str()).await {
                        eprintln!("client message error: {error}");
                        break;
                    }
                }
            }
            event = events.recv() => {
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
}

async fn handle_client_event(
    socket: &mut WebSocket,
    state: &AppState,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event: ClientEvent = serde_json::from_str(text)?;

    match event {
        ClientEvent::Ping => {
            send_event(socket, ServerEvent::Pong).await?;
        }
        ClientEvent::SendMessage { room_id, content } => {
            let message = state.chat_service.send_message(room_id, content);
            let _ = state.events.send(ServerEvent::MessageCreated(message));
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
