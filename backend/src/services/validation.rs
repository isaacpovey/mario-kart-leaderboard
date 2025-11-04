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
            "{field_name} cannot be empty"
        )));
    }

    if trimmed.len() < MIN_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "{field_name} must be at least {MIN_NAME_LENGTH} characters long"
        )));
    }

    if trimmed.len() > MAX_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "{field_name} must be at most {MAX_NAME_LENGTH} characters long"
        )));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || "-_'.".contains(c))
    {
        return Err(AppError::InvalidInput(format!(
            "{field_name} contains invalid characters. Only alphanumeric characters, spaces, hyphens, underscores, apostrophes, and periods are allowed"
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
            "Password must be at least {MIN_PASSWORD_LENGTH} characters long"
        )));
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "Password must be at most {MAX_PASSWORD_LENGTH} characters long"
        )));
    }

    Ok(())
}
