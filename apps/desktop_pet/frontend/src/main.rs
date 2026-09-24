pub mod bridge;

use leptos::mount::mount_to_body;
use leptos::{prelude::*, task::spawn_local};
use pet_domain::{PetCommand, PetSnapshot, PetState};

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

#[component]
fn App() -> impl IntoView {
    let (snapshot, set_snapshot) = signal::<Option<PetSnapshot>>(None);

    let (error, set_error) = signal::<Option<String>>(None);

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
            <PetStatus snapshot />

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
fn PetStatus(snapshot: ReadSignal<Option<PetSnapshot>>) -> impl IntoView {
    let label = move || {
        snapshot
            .get()
            .map(|snapshot| match snapshot.state {
                PetState::Idle => "待機",
                PetState::Interacting => "互動中",
                PetState::Sleeping => "睡覺中",
            })
            .unwrap_or("載入中")
    };

    let revision = move || {
        snapshot
            .get()
            .map(|snapshot| snapshot.revision.to_string())
            .unwrap_or_else(|| "-".into())
    };

    view! {
        <section class="pet-status">
            <div class="pet-face">"🐾"</div>

            <strong>{label}</strong>

            <small>
                "revision "
                {revision}
            </small>
        </section>
    }
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
