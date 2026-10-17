mod bridge;
mod component;
mod duplicate_preview;
mod file_scan_view;

pub use component::*;

use leptos::mount::mount_to_body;
use leptos::{prelude::*, task::spawn_local};
use pet_domain::PetSnapshot;

fn main() {
    mount_to_body(|| {
        view! {
            <App />
        }
    });
}

#[component]
fn App() -> impl IntoView {
    let (snapshot, set_snapshot) = signal::<Option<PetSnapshot>>(None);

    let (error, set_error) = signal::<Option<String>>(None);

    let (blinking, set_blinking) = signal(false);

    let (animation_phase, set_animation_phase) = signal(false);

    let (chat_open, set_chat_open) = signal(false);
    let (chat_input, set_chat_input) = signal(String::new());

    let (chat_loading, set_chat_loading) = signal(false);
    let (chat_response, set_chat_response) = signal::<Option<String>>(None);
    let (chat_error, set_chat_error) = signal::<Option<String>>(None);

    let (duplicate_status, set_duplicate_status) = signal::<Option<String>>(None);
    let (duplicate_loading, set_duplicate_loading) = signal(false);

    let (scan_loading, set_scan_loading) = signal(false);
    let (scan_status, set_scan_status) = signal::<Option<String>>(None);

    // 放在 App 的 signals 區塊；確認只針對目前顯示的掃描結果。
    let (confirmed, set_confirmed) = signal(false);

    {
        let snapshot_reader = snapshot;
        let snapshot_writer = set_snapshot;
        let error_writer = set_error;

        spawn_local(async move {
            let result = bridge::listen_pet_state(move |incoming| {
                apply_snapshot(incoming, snapshot_reader, snapshot_writer);
            })
            .await;

            if let Err(message) = result {
                error_writer.set(Some(message));
            }
        });

        spawn_local(async move {
            match bridge::get_pet_state().await {
                Ok(incoming) => {
                    apply_snapshot(incoming, snapshot_reader, snapshot_writer);
                }

                Err(message) => {
                    error_writer.set(Some(message));
                }
            }
        });
    }

    view! {
        <main class="pet-shell">
            <DragHandle />

            <BlinkController
                snapshot
                set_blinking
            />
            <AnimationClock set_animation_phase />

            <PetStatus snapshot blinking animation_phase />

            <PetControls
                snapshot
                set_snapshot
                set_error
            />

            <button
                class="chat-trigger"
                type="button"
                on:click=move |_| {set_chat_open.set(true);}
            >"聊天"</button>

            <ChatBubble
                 chat_open
                 set_chat_open
                 chat_input
                 set_chat_input
                 chat_loading
                 set_chat_loading
                 chat_response
                 set_chat_response
                 chat_error
                 set_chat_error
            />

            <button
                type="button"
                disabled=move || scan_loading.get() || duplicate_loading.get()
                on:click=move |_| {
                    if scan_loading.get_untracked() || duplicate_loading.get_untracked() { return; }
                    set_scan_loading.set(true);
                    set_scan_status.set(None);
                    set_error.set(None);
                    // 新掃描一律使前一份確認失效，也不保留另一類舊摘要。
                    set_confirmed.set(false);
                    set_scan_status.set(None);
                    set_duplicate_status.set(None);
                    spawn_local(async move {
                        match bridge::scan_file_preview().await {
                            Ok(summary) => set_scan_status.set(Some(summary)),
                            // 不把平台錯誤中的私人路徑顯示於畫面。
                            Err(_) => set_error.set(Some("掃描失敗，請確認已授權目錄後重試。".into())),
                        }
                        set_scan_loading.set(false);
                    });
                }
            >"預覽授權目錄"</button>
            <Show when=move || scan_loading.get() fallback=|| ()>
                <p role="status">"正在掃描授權目錄…"</p>
            </Show>
            <Show when=move || scan_status.get().is_some() fallback=|| ()>
                <p role="status">{move || scan_status.get().unwrap_or_default()}</p>
            </Show>

            <button
                type="button"
                disabled=move || duplicate_loading.get() || scan_loading.get()
                on:click=move |_| {
                    // 清除過時預覽；同時只允許一次使用者請求。
                    if duplicate_loading.get_untracked() || scan_loading.get_untracked()  { return; }
                    set_duplicate_loading.set(true);
                    set_duplicate_status.set(None);
                    set_error.set(None);
                    // 新掃描一律使前一份確認失效，也不保留另一類舊摘要。
                    set_confirmed.set(false);
                    set_scan_status.set(None);
                    set_duplicate_status.set(None);
                    spawn_local(async move {
                        match bridge::scan_duplicate_preview().await {
                            Ok(summary) => set_duplicate_status.set(Some(summary)),
                            Err(message) => set_error.set(Some(message)),
                        }
                        set_duplicate_loading.set(false);
                    });
                }
            >"檢查重複檔案"</button>
            <Show when=move || duplicate_loading.get() fallback=|| ()>
                <p role="status">"正在檢查重複檔案…"</p>
            </Show>
            <Show when=move || duplicate_status.get().is_some() fallback=|| ()>
                <pre class="duplicate-preview" role="status">
                    {move || duplicate_status.get().unwrap_or_default()}
                </pre>
            </Show>

            <button type="button"
                disabled=move || scan_loading.get() || duplicate_loading.get()
                    || (scan_status.get().is_none() && duplicate_status.get().is_none())
                on:click=move |_| set_confirmed.set(true)
            >"確認本次預覽"</button>
            <button type="button"
                disabled=move || scan_loading.get() || duplicate_loading.get()
                on:click=move |_| {
                    set_confirmed.set(false);
                    set_scan_status.set(None);
                    set_duplicate_status.set(None);
                }
            >"取消／清除本次預覽"</button>
            <Show when=move || confirmed.get() fallback=|| ()>
                <p role="status">"已確認本次預覽；尚未執行移動、刪除或還原。"</p>
            </Show>

            <ErrorMessage error />
        </main>
    }
}

#[component]
fn ErrorMessage(error: ReadSignal<Option<String>>) -> impl IntoView {
    view! {
        <Show
            when=move || error.get().is_some()
            fallback=|| ()
        >
            <p class="error">
                {move || {
                    error
                        .get()
                        .unwrap_or_default()
                }}
            </p>
        </Show>
    }
}
