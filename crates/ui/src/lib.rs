mod chat_socket;

use std::rc::Rc;

use chat_domain::ChatMessage;
use leptos::prelude::*;
use shared::{ClientEvent, ServerEvent};
use uuid::Uuid;

use chat_socket::ChatSocket;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main>
            <h1>"Rust AI Chat"</h1>
        </main>
    }
}

#[component]
pub fn ChatPage() -> impl IntoView {
    let room_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("Invalid room ID");

    let (draft, set_draft) = signal(String::new());
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());

    #[cfg(target_arch = "wasm32")]
    let socket = {
        let url = format!("ws://localhost:3000/ws/{room_id}");
        Rc::new(
            ChatSocket::connect(&url, move |event| {
                if let ServerEvent::MessageCreated(message) = event {
                    set_messages.update(|messages| messages.push(message));
                }
            })
            .expect("Failed to connect to WebSocket"),
        )
    };

    #[cfg(not(target_arch = "wasm32"))]
    let socket = Rc::new(ChatSocket::server_stud());

    let send = {
        let socket = Rc::clone(&socket);

        move |_| {
            let content = draft.get_untracked().trim().to_string();

            if content.is_empty() {
                return;
            }

            let event = ClientEvent::SendMessage { room_id, content };

            if socket.send(&event).is_ok() {
                set_draft.set(String::new());
            }
        }
    };

    view! {
        <section>
            <h2>"Room"</h2>
            <ul>
                {move || messages.get().into_iter().map(|message|{
                    view! {
                        <li>
                          <strong>
                            {format!("{:?}: ", message.role)}
                          </strong>
                          {message.content}
                        </li>
                    }}).collect_view()}
            </ul>
            <div>
                <input
                    type="text"
                    placeholder="Type a message..."
                    prop:value=move || draft.get()
                    on:input=move |event| set_draft.set(event_target_value(&event))
                />

                <button type="button" on:click=send>"Send"</button>
            </div>
        </section>
    }
}
