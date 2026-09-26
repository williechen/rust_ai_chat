pub mod bridge;

use gloo_timers::callback::{Interval, Timeout};
use leptos::mount::mount_to_body;
use leptos::{prelude::*, task::spawn_local};
use pet_domain::{PetCommand, PetSnapshot, PetState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetPose {
    Idle,
    Blink,
    Interact,
    Sleep,
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
        .get()
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

fn project_pose(state: PetState, blinking: bool) -> PetPose {
    match state {
        PetState::Idle => {
            if blinking {
                PetPose::Blink
            } else {
                PetPose::Idle
            }
        }
        PetState::Interacting => PetPose::Interact,
        PetState::Sleeping => PetPose::Sleep,
        PetState::Blinking => PetPose::Blink,
    }
}

#[component]
fn App() -> impl IntoView {
    let (snapshot, set_snapshot) = signal::<Option<PetSnapshot>>(None);

    let (error, set_error) = signal::<Option<String>>(None);

    let (blinking, set_blinking) = signal(false);

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

            <PetStatus snapshot blinking/>

            <PetControls
                snapshot
                set_snapshot
                set_error
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
                class="drag-handle-icon"
                data-tauri-drag-region
                aria-hidden="true"
            >
                "..."
            </span>
        </header>
    }
}

#[component]
fn PetStatus(
    snapshot: ReadSignal<Option<PetSnapshot>>,
    blinking: ReadSignal<bool>,
) -> impl IntoView {
    let pose = move || {
        snapshot
            .get()
            .map(|snapshot| project_pose(snapshot.state, blinking.get()))
    };

    let face = move || match pose() {
        Some(PetPose::Idle) => "😺",
        Some(PetPose::Blink) => "😻",
        Some(PetPose::Interact) => "😸",
        Some(PetPose::Sleep) => "😴",
        None => "🐾",
    };

    let label = move || match pose() {
        Some(PetPose::Idle) => "待機",
        Some(PetPose::Blink) => "眨眼",
        Some(PetPose::Interact) => "互動",
        Some(PetPose::Sleep) => "睡覺",
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
            <div class="pet-face">{face()}</div>

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
    fn idle_can_project_to_blink() {
        assert_eq!(project_pose(PetState::Idle, true), PetPose::Blink,);
    }

    #[test]
    fn sleeping_wins_over_blink() {
        assert_eq!(project_pose(PetState::Sleeping, true), PetPose::Sleep,);
    }

    #[test]
    fn interacting_wins_over_blink() {
        assert_eq!(project_pose(PetState::Interacting, true), PetPose::Interact,);
    }
}
