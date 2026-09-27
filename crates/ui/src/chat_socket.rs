use shared::{ClientEvent, ServerEvent};

pub fn decode_server_event(text: &str) -> Result<ServerEvent, serde_json::Error> {
    serde_json::from_str::<ServerEvent>(text)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::{MessageEvent, WebSocket};

    use super::*;

    pub struct ChatSocket {
        socket: WebSocket,
        _onmessage: Closure<dyn FnMut(MessageEvent)>,
    }

    impl ChatSocket {
        pub fn connect<F>(url: &str, mut on_event: F) -> Result<Self, String>
        where
            F: 'static + FnMut(ServerEvent),
        {
            let socket = WebSocket::new(url)
                .map_err(|err| format!("Failed to create WebSocket: {:?}", err))?;

            let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
                let Some(text) = event.data().as_string() else {
                    return;
                };
                let Ok(event) = decode_server_event(&text) else {
                    return;
                };
                on_event(event);
            }) as Box<dyn FnMut(MessageEvent)>);

            socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

            Ok(Self {
                socket,
                _onmessage: onmessage,
            })
        }

        pub fn send(&self, event: &ClientEvent) -> Result<(), String> {
            let json = serde_json::to_string(event)
                .map_err(|err| format!("Failed to serialize event: {:?}", err))?;
            self.socket
                .send_with_str(&json)
                .map_err(|err| format!("Failed to send event: {:?}", err))
        }
    }

    impl Drop for ChatSocket {
        fn drop(&mut self) {
            self.socket.set_onmessage(None);
            let _ = self.socket.close();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::ChatSocket;

#[cfg(not(target_arch = "wasm32"))]
pub struct ChatSocket;

#[cfg(not(target_arch = "wasm32"))]
impl ChatSocket {
    pub fn server_stud() -> Self {
        Self
    }

    pub fn send(&self, _event: &ClientEvent) -> Result<(), String> {
        Err("ChatSocket is not supported on this platform".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_pong_event() {
        let event =
            decode_server_event(r#"{"type":"pong"}"#).expect("Failed to decode server event");

        assert!(matches!(event, ServerEvent::Pong));
    }
}
