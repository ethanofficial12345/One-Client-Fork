use chrono::{Duration, Utc};
use md5::{Digest, Md5};
use uuid::Uuid;

use super::data::{AccountKind, MinecraftAccount};
use super::error::AuthError;
use crate::error::AuthResult;

pub fn offline_uuid(username: &str) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{username}").as_bytes());
    let digest = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub fn validate_offline_username(username: &str) -> AuthResult<()> {
    let len = username.chars().count();
    if !(3..=16).contains(&len) {
        return Err(AuthError::InvalidOfflineUsername {
            reason: "username must be 3-16 characters".into(),
        });
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AuthError::InvalidOfflineUsername {
            reason: "username may only contain letters, digits, and underscores".into(),
        });
    }

    Ok(())
}

#[tracing::instrument(level = "debug", fields(username = %username))]
pub fn offline_account(username: String) -> MinecraftAccount {
    tracing::info!("creating offline account");
    MinecraftAccount {
        id: offline_uuid(&username),
        username,
        access_token: String::new(),
        refresh_token: String::new(),
        expires: Utc::now() + Duration::days(3650),
        kind: AccountKind::Offline,
        signed_out: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuid_matches_the_vanilla_algorithm() {
        // Known vanilla value: MD5("OfflinePlayer:TestUser") with version 3
        let expected = Uuid::parse_str("097d3392-865a-3f3c-8b4a-da1c3473466c").unwrap();

        let first = offline_uuid("TestUser");
        let second = offline_uuid("TestUser");

        assert_eq!(first, expected);
        assert_eq!(first, second, "the same name must always map to one uuid");
        assert_eq!(first.get_version_num(), 3);
        assert_eq!(first.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn a_two_character_username_is_rejected() {
        assert!(matches!(
            validate_offline_username("ab"),
            Err(AuthError::InvalidOfflineUsername { .. })
        ));
    }

    #[test]
    fn a_seventeen_character_username_is_rejected() {
        assert!(matches!(
            validate_offline_username("abcdefghijklmnopq"),
            Err(AuthError::InvalidOfflineUsername { .. })
        ));
    }

    #[test]
    fn a_hyphenated_username_is_rejected() {
        assert!(matches!(
            validate_offline_username("bad-name"),
            Err(AuthError::InvalidOfflineUsername { .. })
        ));
    }

    #[test]
    fn a_vanilla_username_is_accepted() {
        assert!(validate_offline_username("Steve").is_ok());
        assert!(validate_offline_username("Player_123").is_ok());
    }
}
