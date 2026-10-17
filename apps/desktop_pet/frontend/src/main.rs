mod bridge;
mod component;
mod duplicate_preview;
mod file_scan_view;

pub use component::*;

use leptos::mount::mount_to_body;
use leptos::{prelude::*, task::spawn_local};
use pet_domain::PetSnapshot;

fn accept_scan_result(request_generation: u64, current_generation: u64) -> bool {
    request_generation == current_generation
}

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
    let (generation, set_generation) = signal(0_u64);

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

    if bridge::is_pet_window() {
        return view! {
            <main class="pet-life">
                <DragHandle />
                <BlinkController
                    snapshot
                    set_blinking
                />
                <AnimationClock set_animation_phase />
                <PetStatus snapshot blinking animation_phase />
                <button
                    type="button"
                    on:click=move |_| spawn_local(async move{
                        if bridge::open_controls().await.is_err(){
                            set_error.set(Some("無法開啟控制台".into()));
                        }
                    })
                >"確認"</button>
            </main>
        }
        .into_any();
    }

    view! {
        <main class="control-shell">
            <h1>"寵物控制台"</h1>
            <p>"控制台關閉後寵物仍運作；檔案功能目前只提供唯讀預覽"</p>

            <h2>"互動與聊天"</h2>
            <p>{move || snapshot.get().map(|s| format!("目前狀態: {:?}, 最近更新: {:?}", s.state, s.revision)).unwrap_or_else(|| "讀取中…".into())}</p>

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

            <h2>"檔案管理"</h2>
            <button
                type="button"
                disabled=move || scan_loading.get() || duplicate_loading.get()
                on:click=move |_| {
                    if scan_loading.get_untracked() || duplicate_loading.get_untracked() { return; }
                    let request = generation.get_untracked().wrapping_add(1);
                    set_generation.set(request);
                    set_scan_loading.set(true);
                    set_scan_status.set(None);
                    set_error.set(None);
                    // 新掃描一律使前一份確認失效，也不保留另一類舊摘要。
                    set_confirmed.set(false);
                    set_scan_status.set(None);
                    set_duplicate_status.set(None);
                    spawn_local(async move {
                        let result = bridge::scan_file_preview().await;
                        set_scan_loading.set(false);
                        if !accept_scan_result(request, generation.get_untracked()) {
                            return;
                        }
                        match result {
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
                    let request = generation.get_untracked().wrapping_add(1);
                    set_generation.set(request);
                    set_duplicate_loading.set(true);
                    set_duplicate_status.set(None);
                    set_error.set(None);
                    // 新掃描一律使前一份確認失效，也不保留另一類舊摘要。
                    set_confirmed.set(false);
                    set_scan_status.set(None);
                    set_duplicate_status.set(None);
                    spawn_local(async move {
                        let result = bridge::scan_duplicate_preview().await;
                        set_duplicate_loading.set(false);
                        if !accept_scan_result(request, generation.get_untracked()) {
                            return;
                        }
                        match result {
                            Ok(summary) => set_duplicate_status.set(Some(summary)),
                            Err(_) => set_error.set(Some("預覽失敗，請確認授權目錄後重試。".into())),
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

            <button 
                type="button"
                on:click=move |_| spawn_local(async move {
                    let _ = bridge::quit_pet().await;
                })
            >"寵物回寵了"</button>
        </main>
    }.into_any()
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

#[cfg(test)]
mod day16_scan_generation_tests {
    use super::accept_scan_result;

    #[test]
    fn canceled_or_superseded_scan_cannot_restore_preview() {
        assert!(accept_scan_result(3, 3));
        assert!(!accept_scan_result(2, 3));
        assert!(!accept_scan_result(3, 4));
    }
}
