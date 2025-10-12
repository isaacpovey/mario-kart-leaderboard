use crate::error::{AppError, Result};

const MIN_NAME_LENGTH: usize = 1;
const MAX_NAME_LENGTH: usize = 100;
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

/// Validates a name (for players, groups, etc.)
pub fn validate_name(name: &str, field_name: &str) -> Result<()> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "{} cannot be empty",
            field_name
        )));
    }

    if trimmed.len() < MIN_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "{} must be at least {} characters long",
            field_name, MIN_NAME_LENGTH
        )));
    }

    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "{} must be at most {} characters long",
            field_name, MAX_NAME_LENGTH
        )));
    }

    // Check for invalid characters (optional, adjust based on requirements)
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || "-_'.".contains(c))
    {
        return Err(AppError::InvalidInput(format!(
            "{} contains invalid characters. Only alphanumeric characters, spaces, hyphens, underscores, apostrophes, and periods are allowed",
            field_name
        )));
    }

    Ok(())
}

/// Validates a password
pub fn validate_password(password: &str) -> Result<()> {
    if password.is_empty() {
        return Err(AppError::InvalidInput(
            "Password cannot be empty".to_string(),
        ));
    }

    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        )));
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "Password must be at most {} characters long",
            MAX_PASSWORD_LENGTH
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_success() {
        assert!(validate_name("John Doe", "Name").is_ok());
        assert!(validate_name("Player-1", "Name").is_ok());
        assert!(validate_name("O'Brien", "Name").is_ok());
        assert!(validate_name("Test_User", "Name").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("", "Name").is_err());
        assert!(validate_name("   ", "Name").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(validate_name(&long_name, "Name").is_err());
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(validate_name("Test@User", "Name").is_err());
        assert!(validate_name("User#123", "Name").is_err());
    }

    #[test]
    fn test_validate_password_success() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("SuperSecret!@#").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        assert!(validate_password("short").is_err());
    }

    #[test]
    fn test_validate_password_empty() {
        assert!(validate_password("").is_err());
    }
}
