//! Input validation helpers.

use crate::error::AppError;

const MAX_USERNAME_LEN: usize = 20;

/// Validate a display username: 1–20 chars, must start with a letter or digit,
/// and only contain `A-Z a-z 0-9 . _ -` ("normal" username specials). Used for
/// admin renames and as the base check for registration.
pub fn validate_username(name: &str) -> Result<(), AppError> {
    let len = name.chars().count();
    if len == 0 || len > MAX_USERNAME_LEN {
        return Err(AppError::BadRequest(format!(
            "username must be 1–{MAX_USERNAME_LEN} characters"
        )));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Err(AppError::BadRequest(
            "username must start with a letter or digit".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(AppError::BadRequest(
            "username may only contain letters, digits, and . _ -".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid() {
        assert!(validate_username("Alice_99").is_ok());
        assert!(validate_username("a.b-c").is_ok());
    }

    #[test]
    fn rejects_bad() {
        assert!(validate_username("").is_err());
        assert!(validate_username(&"a".repeat(21)).is_err());
        assert!(validate_username("_leading").is_err());
        assert!(validate_username("bad space").is_err());
        assert!(validate_username("inject;drop").is_err());
    }
}
