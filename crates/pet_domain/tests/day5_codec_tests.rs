const VALID: &str =
    r#"{"名字":"小啾","種類":"鳥","喜好":["水果"],"個性":["活潑"],"興趣":["音樂"]}"#;

#[test]
fn valid_five_fields_round_trip() {
    let p = decode_profile(VALID).unwrap();
    let encoded = encode_profile(&p).unwrap();
    let decoded = decode_profile(&encoded).unwrap();
    assert_eq!(decoded, p);
    assert!(decoded.appearance.is_none());
}

#[test]
fn rejects_blank_name_on_decode() {
    let bad = VALID.replace("小啾", " ");
    assert!(matches!(
        decode_profile(&bad),
        Err(ProfileCodecError::Invalid(_))
    ));
}

#[test]
fn rejects_invalid_json() {
    assert!(matches!(
        decode_profile("{"),
        Err(ProfileCodecError::Json(_))
    ));
}

#[test]
fn optional_appearance_is_preserved() {
    let json =
        r#"{"名字":"小啾","種類":"鳥","喜好":[],"個性":[],"興趣":[],"外觀":{"顏色":"綠色"}}"#;
    let p = decode_profile(json).unwrap();
    let encoded = encode_profile(&p).unwrap();
    let again: PetProfile = decode_profile(&encoded).unwrap();
    assert_eq!(again.appearance.unwrap().color.as_deref(), Some("綠色"));
}

#[test]
fn invalid_profile_cannot_be_encoded() {
    let mut p = decode_profile(VALID).unwrap();
    p.species = " ".into();
    assert!(matches!(
        encode_profile(&p),
        Err(ProfileCodecError::Invalid(_))
    ));
}
