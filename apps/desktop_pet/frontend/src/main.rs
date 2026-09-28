pub mod bridge;

use gloo_timers::callback::{Interval, Timeout};
use leptos::mount::mount_to_body;
use leptos::{prelude::*, task::spawn_local};
use pet_domain::{PetCommand, PetSnapshot, PetState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetPose {
    Idle,
    Blink,
    InteractA,
    InteractB,
    SleepA,
    SleepB,
}

fn main() {
    mount_to_body(|| {
        view! {
            <App />
        }
    });
}

fn apply_snapshot(
    incoming: PetSnapshot,
    snapshot: ReadSignal<Option<PetSnapshot>>,
    set_snapshot: WriteSignal<Option<PetSnapshot>>,
) {
    let should_apply = snapshot
        .get_untracked()
        .map(|current| incoming.revision >= current.revision)
        .unwrap_or(true);

    if should_apply {
        set_snapshot.set(Some(incoming));
    }
}

fn dispatch_command(
    command: PetCommand,
    snapshot: ReadSignal<Option<PetSnapshot>>,
    set_snapshot: WriteSignal<Option<PetSnapshot>>,
    set_error: WriteSignal<Option<String>>,
) {
    set_error.set(None);

    spawn_local(async move {
        match bridge::send_pet_command(command).await {
            Ok(incoming) => {
                apply_snapshot(incoming, snapshot, set_snapshot);
            }

            Err(message) => {
                set_error.set(Some(message));
            }
        }
    });
}

fn project_pose(state: PetState, blinking: bool, animation_phase: bool) -> PetPose {
    match state {
        PetState::Idle => {
            if blinking {
                PetPose::Blink
            } else {
                PetPose::Idle
            }
        }
        PetState::Interacting => {
            if animation_phase {
                PetPose::InteractB
            } else {
                PetPose::InteractA
            }
        }
        PetState::Sleeping => {
            if animation_phase {
                PetPose::SleepB
            } else {
                PetPose::SleepA
            }
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let (snapshot, set_snapshot) = signal::<Option<PetSnapshot>>(None);

    let (error, set_error) = signal::<Option<String>>(None);

    let (blinking, set_blinking) = signal(false);

    let (animation_phase, set_animation_phase) = signal(false);

    let (chat_open, set_chat_open) = signal(false);
    let (chat_input, set_chat_input) = signal(String::new());

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
            />

            <ErrorMessage error />
        </main>
    }
}

#[component]
fn DragHandle() -> impl IntoView {
    view! {
        <header
            class="drag-handle"
            data-tauri-drag-region
            aria-label="拖曳桌面小寵物"
        >
            <span
                class="drag-grip"
                data-tauri-drag-region
                aria-hidden="true"
            >
                "..."
            </span>
        </header>
    }
}

#[component]
fn AnimationClock(set_animation_phase: WriteSignal<bool>) -> impl IntoView {
    let interval = Interval::new(650, move || {
        set_animation_phase.update(|phase| *phase = !*phase);
    });
    let _interval = StoredValue::new_local(interval);

    ().into_any()
}

#[component]
fn PetStatus(
    snapshot: ReadSignal<Option<PetSnapshot>>,
    blinking: ReadSignal<bool>,
    animation_phase: ReadSignal<bool>,
) -> impl IntoView {
    let pose = move || {
        snapshot
            .get()
            .map(|snapshot| project_pose(snapshot.state, blinking.get(), animation_phase.get()))
    };

    let face = Signal::derive(move || match pose() {
        Some(PetPose::Idle) => "😺",
        Some(PetPose::Blink) => "😻",
        Some(PetPose::InteractA) => "😸",
        Some(PetPose::InteractB) => "😹",
        Some(PetPose::SleepA) => "😴",
        Some(PetPose::SleepB) => "😪",
        None => "🐾",
    });

    let label = move || match pose() {
        Some(PetPose::Idle) => "待機",
        Some(PetPose::Blink) => "眨眼",
        Some(PetPose::InteractA | PetPose::InteractB) => "互動",
        Some(PetPose::SleepA | PetPose::SleepB) => "睡覺",
        None => "載入中",
    };

    let revision = move || {
        snapshot
            .get()
            .map(|snapshot| snapshot.revision.to_string())
            .unwrap_or_else(|| "-".into())
    };

    view! {
        <section class="pet-status">
            <div
                class="pet-face"

                on:contextmenu=move |event| {
                    event.prevent_default();

                    let x = event.client_x();
                    let y = event.client_y();

                    spawn_local(async move {
                        if let Err(error) =
                            bridge::show_pet_context_menu(x, y).await
                        {
                            web_sys::console::warn_1(
                                &error.into()
                            );
                        }
                    });
                }
            >
                {face}
            </div>

            <strong>{label}</strong>

            <small>
                "revision "
                {revision}
            </small>
        </section>
    }
}

#[component]
fn BlinkController(
    snapshot: ReadSignal<Option<PetSnapshot>>,
    set_blinking: WriteSignal<bool>,
) -> impl IntoView {
    let interval = Interval::new(4_000, move || {
        let is_idle = snapshot
            .get_untracked()
            .map(|snapshot| snapshot.state == PetState::Idle)
            .unwrap_or(false);

        if !is_idle {
            set_blinking.set(false);
            return;
        }

        set_blinking.set(true);

        Timeout::new(180, move || {
            set_blinking.set(false);
        })
        .forget();
    });

    // gloo_timers::Interval 是 !Send / !Sync。
    // StoredValue::new_local() 會把它保存在目前 Leptos owner 的 local arena，
    // owner dispose 時一起 drop，Interval 也會因此取消。
    let _interval = StoredValue::new_local(interval);

    ().into_any()
}

#[component]
fn PetControls(
    snapshot: ReadSignal<Option<PetSnapshot>>,
    set_snapshot: WriteSignal<Option<PetSnapshot>>,
    set_error: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="pet-controls">
            <button
                on:click=move |_| {
                    dispatch_command(
                        PetCommand::Interact,
                        snapshot,
                        set_snapshot,
                        set_error,
                    );
                }
            >
                "互動"
            </button>

            <button
                on:click=move |_| {
                    dispatch_command(
                        PetCommand::FinishInteraction,
                        snapshot,
                        set_snapshot,
                        set_error,
                    );
                }
            >
                "結束"
            </button>

            <button
                on:click=move |_| {
                    dispatch_command(
                        PetCommand::Sleep,
                        snapshot,
                        set_snapshot,
                        set_error,
                    );
                }
            >
                "睡覺"
            </button>

            <button
                on:click=move |_| {
                    dispatch_command(
                        PetCommand::Wake,
                        snapshot,
                        set_snapshot,
                        set_error,
                    );
                }
            >
                "醒來"
            </button>
        </section>
    }
}

#[component]
fn ChatBubble(
    chat_open: ReadSignal<bool>,
    set_chat_open: WriteSignal<bool>,
    chat_input: ReadSignal<String>,
    set_chat_input: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <Show
            when=move || chat_open.get()
            fallback=|| ()
        >
            <section class="chat-bubble">
                <div class="chat-bubble__header">
                    <strong>"跟小寵物說話"</strong>

                    <button
                        type="button"
                        class="chat-bubble__close"
                        aria-label="關閉對話"
                        on:click=move |_| {
                            set_chat_open.set(false);
                        }
                    >
                        "×"
                    </button>
                </div>

                <input
                    class="chat-bubble__input"
                    type="text"
                    placeholder="輸入一句話…"

                    prop:value=move || {
                        chat_input.get()
                    }

                    on:input=move |event| {
                        set_chat_input.set(
                            event_target_value(&event)
                        );
                    }
                />

                <button
                    class="chat-bubble__send"
                    type="button"
                    prop:disabled=true
                >
                    "送出（Day 11）"
                </button>
            </section>
        </Show>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_blink_has_priority() {
        assert_eq!(project_pose(PetState::Idle, true, true,), PetPose::Blink,);
    }

    #[test]
    fn interacting_uses_animation_phase() {
        assert_eq!(
            project_pose(PetState::Interacting, false, false,),
            PetPose::InteractA,
        );

        assert_eq!(
            project_pose(PetState::Interacting, false, true,),
            PetPose::InteractB,
        );
    }

    #[test]
    fn sleeping_uses_animation_phase() {
        assert_eq!(
            project_pose(PetState::Sleeping, false, false,),
            PetPose::SleepA,
        );

        assert_eq!(
            project_pose(PetState::Sleeping, false, true,),
            PetPose::SleepB,
        );
    }
}
