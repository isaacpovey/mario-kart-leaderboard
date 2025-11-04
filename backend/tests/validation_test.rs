use mario_kart_leaderboard_backend::services::validation::*;

const MAX_NAME_LENGTH: usize = 100;

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
