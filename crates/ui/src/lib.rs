mod chat_socket;

use std::rc::Rc;

use chat_domain::{ChatMessage, MessageAuthor};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::{use_navigate, use_params_map},
    path,
};
use shared::{ClientEvent, ServerEvent};
use uuid::Uuid;

use chat_socket::ChatSocket;

fn author_label(author: &MessageAuthor) -> String {
    match author {
        MessageAuthor::AnonymousUser => "You".to_string(),
        MessageAuthor::User { user_id } => format!("User {}", user_id),
        MessageAuthor::Ai { persona_id } => format!("AI {}", persona_id),
        MessageAuthor::System => "System".to_string(),
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"Rust AI Chat"</title>
                <HydrationScripts options />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <h1>"Rust AI Chat"</h1>
                <Routes fallback=|| view!{
                    <p>"404 - Page Not Found"</p>
                }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/room/:room_id") view=RoomPage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <p>"Welcome to Rust AI Chat!"</p>
        <button
            type="button"
            on:click=move |_| {
                let room_id = Uuid::new_v4();
                let path = format!("/room/{}", room_id);
                navigate(&path, Default::default());
            }
        >"Create Room"</button>
    }
}

#[component]
fn RoomPage() -> impl IntoView {
    let params = use_params_map();

    move || {
        let room_id = params
            .read()
            .get("room_id")
            .and_then(|value| Uuid::parse_str(&value).ok());

        match room_id {
            Some(room_id) => view! { <ChatPage room_id /> }.into_any(),
            None => view! { <p>"Invalid Room Id"</p> }.into_any(),
        }
    }
}

#[component]
pub fn ChatPage(room_id: String) -> impl IntoView {
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
    let socket = Rc::new(ChatSocket::server_stub());

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
                            {author_label(&message.author)}": "
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
