pub mod activity;
pub mod chat;
pub mod control;

use leptos::prelude::*;
use pet_domain::PetSnapshot;

pub use activity::*;
pub use chat::*;
pub use control::*;

// 應用程式快照
pub fn apply_snapshot(
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
