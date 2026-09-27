use leptos::prelude::*;

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
    let (draft, set_draft) = signal(String::new());
    let (messages, set_messages) = signal(Vec::<String>::new());

    let send = move |_| {
        let content = draft.get().trim().to_string();

        if content.is_empty() {
            return;
        }

        set_messages.update(|messages| messages.push(content));
        set_draft.set(String::new());
    };

    view! {
        <section>
            <h2>"Room"</h2>
            <ul>
                {move || messages.get().into_iter().map(|message| view! { <li>{message}</li> }).collect_view()}
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
