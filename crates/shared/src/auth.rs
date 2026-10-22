use sha2::{Digest, Sha256};

pub fn cookie_token(header: &str, name: &str) -> Option<String> {
    let value = header
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| *key == name)?
        .1;
    (value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())).then(|| value.to_owned())
}

pub fn token_digest(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cookie_name_is_exact_and_token_is_validated() {
        let token = "a".repeat(64);
        assert_eq!(
            cookie_token(
                &format!("xchat_session={token}; chat_session={token}"),
                "chat_session"
            ),
            Some(token.clone())
        );
        assert_eq!(cookie_token("chat_session=bad", "chat_session"), None);
        assert_eq!(cookie_token("analytics_session=abc", "chat_session"), None);
        assert_eq!(token_digest(&token).len(), 64);
    }
}
