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
                <style>{include_str!("chat.css")}</style>
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
            <main class="app">
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
            Some(room_id) => view! {
                <div class="chat-layout">
                    <RoomList />
                    <ChatPage room_id />
                </div>
            }
            .into_any(),
            None => view! { <p>"Invalid Room Id"</p> }.into_any(),
        }
    }
}

#[component]
pub fn ChatPage(room_id: String) -> impl IntoView {
    let (draft, set_draft) = signal(String::new());

    let (connection, set_connection) = signal(String::from("尚未連線"));
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());

    let heading = room_id.clone();

    #[cfg(not(target_arch = "wasm32"))]
    let _ = set_messages;

    #[cfg(target_arch = "wasm32")]
    let socket = {
        let location = web_sys::window().unwrap().location();
        let protocol = if location.protocol().unwrap() == "https:" {
            "wss"
        } else {
            "ws"
        };
        let url = format!("{protocol}://{}/ws/{room_id}", location.host().unwrap());
        match ChatSocket::connect(
            &url,
            move |event| {
                if let ServerEvent::MessageCreated(message) = event {
                    set_messages.update(|messages| messages.push(message));
                }
            },
            move |status| set_connection.set(status.into()),
        ) {
            Ok(socket) => Some(Rc::new(socket)),
            Err(_) => {
                set_connection.set("連線失敗，請重新進房".into());
                None
            }
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let socket = Some(Rc::new(ChatSocket::server_stub()));

    let send = {
        let socket = socket.clone();

        move |_| {
            let room_id = room_id.clone();

            let content = draft.get_untracked().trim().to_string();

            if content.is_empty() {
                return;
            }

            let event = ClientEvent::SendMessage { room_id, content };

            match socket
                .as_ref()
                .ok_or_else(|| "尚未連線".to_string())
                .and_then(|socket| socket.send(&event).map_err(|_| "發送失敗".to_string()))
            {
                Ok(()) => {
                    set_draft.set(String::new());
                    set_connection.set("已發送，等待回應...".into());
                }
                Err(_) => set_connection.set("發送失敗，請重新進房".into()),
            }
        }
    };

    view! {
        <section class="conversation">
            <header>
                <h2>{format!("房間 {heading}")}</h2>
                <p role="status">{move || connection.get()}</p>
            </header>
            <ul class="messages" aria-live="polite">
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
                <textarea
                    aria-label="聊天訊息"
                    placeholder="Type a message..."
                    prop:value=move || draft.get()
                    on:input=move |event| set_draft.set(event_target_value(&event))
                />

                <button
                    type="button"
                    disabled=move || draft.get().trim().is_empty()
                    on:click=send>"Send"</button>
            </div>
        </section>
    }
}

#[component]
fn RoomList() -> impl IntoView {
    let params = use_params_map();
    let (query, set_query) = signal(String::new());
    let (rows, set_rows) = signal(Vec::<(String, String)>::new());
    let (loading, set_loading) = signal(false);
    let (status, set_status) = signal(String::from("請讀取房間列表"));

    view! {
        <section>
            <h2>"房間列表"</h2>
            <a href="/" rel="external">"首頁"</a>
            <input
                type="search"
                aria-label="搜尋房間"
                placeholder="房間名稱"
                prop:value=move || query.get()
                on:input=move |event| set_query.set(event_target_value(&event))
            />
            <button
                type="button"
                disabled=move || loading.get()
                on:click=move |_| {
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

fn filter_rooms(rows: &[(String, String)], query: &str) -> Vec<(String, String)> {
    let needle = query.trim().to_lowercase();
    rows.iter()
        .filter(|(_, name)| name.to_lowercase().contains(&needle))
        .cloned()
        .collect()
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

#[cfg(all(test, feature = "ssr"))]
mod day16_layout_tests {
    use super::*;
    #[test]
    fn room_page_has_real_room_identity_and_multiline_composer() {
        let owner = Owner::new();
        let html =
            owner.with(|| view! { <ChatPage room_id="room-fixture".to_owned() /> }.to_html());
        assert!(html.contains("room-fixture"));
        assert!(html.contains("<textarea"));
        assert!(html.contains("尚未連線"));
        assert!(html.contains("歷史載入尚未接通"));
        assert!(!html.contains("Online: 100"));
    }
}
