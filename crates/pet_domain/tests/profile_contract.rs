use pet_domain::profile::PetProfile;

fn sample() -> &'static str {
    r#"{"名字":"小啾","種類":"鳥",
         "喜好":["唱歌","水果"],"個性":["吵雜"],
         "興趣":["音樂","探索"]}"#
}

#[test]
fn only_five_fields_are_enough() {
    let p: PetProfile = serde_json::from_str(sample()).unwrap();
    p.validate().unwrap();
    assert_eq!(p.name, "小啾");
    assert_eq!(p.species, "鳥");
    assert!(p.appearance.is_none());
}

#[test]
fn optional_override_keeps_other_fields_auto() {
    let json = r#"{"名字":"小啾","種類":"鳥","喜好":["水果"],
      "個性":["吵雜"],"興趣":["探索"],"外觀":{"顏色":"綠色"}}"#;
    let p: PetProfile = serde_json::from_str(json).unwrap();
    p.validate().unwrap();
    let a = p.appearance.unwrap();
    assert_eq!(a.color.as_deref(), Some("綠色"));
    assert!(a.style.is_none());
}

#[test]
fn missing_or_blank_fields_fail() {
    let missing = r#"{"名字":"小啾","種類":"鳥","喜好":[],"個性":[]}"#;
    assert!(serde_json::from_str::<PetProfile>(missing).is_err());
    let mut p: PetProfile = serde_json::from_str(sample()).unwrap();
    p.name = " ".into();
    assert!(p.validate().is_err());
    p.name = "小啾".into();
    p.interests.push(" ".into());
    assert!(p.validate().is_err());
}

#[test]
fn species_is_extensible() {
    let json = sample().replace("\"鳥\"", "\"龜\"");
    let p: PetProfile = serde_json::from_str(&json).unwrap();
    p.validate().unwrap();
    assert_eq!(p.species, "龜");
}
