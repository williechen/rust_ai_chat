use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProfileError {
    #[error("名字不可空白")]
    EmptyName,
    #[error("種類不可空白")]
    EmptySpecies,
    #[error("喜好、個性、興趣不可包含空白項目")]
    EmptyTag,
    #[error("外觀覆寫不可為空白")]
    EmptyOverride,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceOverride {
    #[serde(rename = "顏色", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "風格", default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PetProfile {
    #[serde(rename = "名字")]
    pub name: String,
    #[serde(rename = "種類")]
    pub species: String,
    #[serde(rename = "喜好")]
    pub likes: Vec<String>,
    #[serde(rename = "個性")]
    pub personality: Vec<String>,
    #[serde(rename = "興趣")]
    pub interests: Vec<String>,
    #[serde(rename = "外觀", default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<AppearanceOverride>,
}

impl PetProfile {
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.name.trim().is_empty() {
            return Err(ProfileError::EmptyName);
        }
        if self.species.trim().is_empty() {
            return Err(ProfileError::EmptySpecies);
        }
        if self
            .likes
            .iter()
            .chain(self.personality.iter())
            .chain(self.interests.iter())
            .any(|s| s.trim().is_empty())
        {
            return Err(ProfileError::EmptyTag);
        }
        if let Some(a) = &self.appearance
            && (a.color.as_deref().is_some_and(|s| s.trim().is_empty())
                || a.style.as_deref().is_some_and(|s| s.trim().is_empty()))
        {
            return Err(ProfileError::EmptyOverride);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileCodecError {
    #[error("invalid profile: {0}")]
    Invalid(#[from] ProfileError),
    #[error("json codec error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn decode_profile(input: &str) -> Result<PetProfile, ProfileCodecError> {
    let profile: PetProfile = serde_json::from_str(input)?;
    profile.validate()?;
    Ok(profile)
}

pub fn encode_profile(profile: &PetProfile) -> Result<String, ProfileCodecError> {
    profile.validate()?;
    Ok(serde_json::to_string_pretty(profile)?)
}
