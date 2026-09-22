fn main() {
    let state = pet_domain::PetState::Idle;
    let next = state.on_event(&pet_domain::PetEvent::UserClicked);

    println!("pet state: {:?}", next);
}
