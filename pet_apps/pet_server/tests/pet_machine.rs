use pet_domain::{PetCommand, PetMachine, PetSnapshot, PetState, error::PetError};

#[test]
fn pet_starts_idle() {
    let pet = PetMachine::default();

    assert_eq!(
        pet.snapshot(),
        PetSnapshot {
            state: PetState::Idle,
            revision: 0,
        }
    );
}

#[test]
fn interact_moves_idle_to_interacting() {
    let mut pet = PetMachine::default();

    let snapshot = pet.dispatch(PetCommand::Interact).unwrap();

    assert_eq!(snapshot.state, PetState::Interacting);
    assert_eq!(snapshot.revision, 1);
}

#[test]
fn sleeping_pet_cannot_interact() {
    let mut pet = PetMachine::default();

    pet.dispatch(PetCommand::Sleep).unwrap();

    let before = pet.snapshot();

    let result = pet.dispatch(PetCommand::Interact);

    assert!(matches!(result, Err(PetError::InvalidTransition { .. })));

    assert_eq!(pet.snapshot(), before);
}

#[test]
fn wake_moves_sleeping_pet_back_to_idle() {
    let mut pet = PetMachine::default();

    pet.dispatch(PetCommand::Sleep).unwrap();

    let snapshot = pet.dispatch(PetCommand::Wake).unwrap();

    assert_eq!(snapshot.state, PetState::Idle);
    assert_eq!(snapshot.revision, 2);
}
