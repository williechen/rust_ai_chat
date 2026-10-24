use crate::bridge;
use leptos::prelude::*;
use pet_domain::{PetCommand, PetSnapshot};
use wasm_bindgen_futures::spawn_local;

use crate::component::apply_snapshot;

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
pub fn PetControls(
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
