mod chat_socket;

#[cfg(target_arch = "wasm32")]
mod room_api;

use std::rc::Rc;

use chat_domain::{ChatMessage, MessageAuthor};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_params_map,
    path,
};

use shared::ClientEvent;

#[cfg(target_arch = "wasm32")]
use shared::ServerEvent;

#[cfg(target_arch = "wasm32")]
use leptos_router::hooks::use_navigate;

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
    #[cfg(target_arch = "wasm32")]
    let navigate = use_navigate();

    let (creating, set_creating) = signal(false);
    let (create_error, set_create_error) = signal(None::<String>);

    let create_room = move |_| {
        if creating.get_untracked() {
            return;
        }

        set_creating.set(true);
        set_create_error.set(None);

        #[cfg(target_arch = "wasm32")]
        {
            let navigate = navigate.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match room_api::create_room("New Room").await {
                    Ok(room) => {
                        let path = format!("/room/{}", room.id);
                        navigate(&path, Default::default());
                    }
                    Err(error) => {
                        set_create_error.set(Some(error));
                        set_creating.set(false);
                    }
                }
            });
        }
    };

    view! {
        <p>"Welcome to Rust AI Chat!"</p>
        <button
            type="button"
            disabled = move || creating.get()
            on:click=create_room
        >{move || {
            if creating.get() {
                "Creating..."
            } else {
                "Create Room"
            }
        }}
        </button>
        <RoomList />

        {move || create_error.get().map(|message| view! { <p>{message}</p> })}
    }
}

#[component]
fn RoomPage() -> impl IntoView {
    let params = use_params_map();

    move || {
        let room_id = params.read().get("room_id");

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

    #[cfg(not(target_arch = "wasm32"))]
    let _ = set_messages;

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
            let room_id = room_id.clone();

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

#[component]
fn RoomList() -> impl IntoView {
    let (rows, set_rows) = signal(Vec::<(String, String)>::new());
    let (loading, set_loading) = signal(false);
    let (status, set_status) = signal(String::from("請讀取房間列表"));
    view! {
        <section>
            <h2>"房間列表"</h2>
            <button type="button" disabled=move || loading.get() on:click=move |_| {
                if loading.get_untracked() { return; }
                set_loading.set(true);
                set_rows.set(Vec::new());
                set_status.set("載入中…".into());
                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(async move {
                    match room_api::list_rooms().await {
                        Ok(value) => {
                            set_status.set(if value.is_empty() { "尚無房間" } else { "請選擇房間" }.into());
                            set_rows.set(value);
                        }
                        Err(message) => set_status.set(message),
                    }
                    set_loading.set(false);
                });
                #[cfg(not(target_arch = "wasm32"))]
                { set_loading.set(false); }
            }>"讀取／重新整理房間"</button>
            <p role="status">{move || status.get()}</p>
            <ul>{move || rows.get().into_iter().map(|(id, name)| view! {
                <li><a href=format!("/room/{id}")>{name}</a></li>
            }).collect_view()}</ul>
        </section>
    }
}

#[cfg(test)]
mod day16_room_search_tests {
    use super::filter_rooms;

    #[test]
    fn search_matches_chinese_names_and_preserves_ids() {
        let rooms = vec![
            ("room_1".to_owned(), "Rust 技術交流".to_owned()),
            ("room_2".to_owned(), "生活閒聊".to_owned()),
        ];
        assert_eq!(filter_rooms(&rooms, " 技術 "), vec![rooms[0].clone()]);
        assert_eq!(filter_rooms(&rooms, ""), rooms);
        assert!(filter_rooms(&rooms, "不存在").is_empty());
    }
}
