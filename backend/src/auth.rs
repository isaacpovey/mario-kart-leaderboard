use crate::error::{AppError, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const JWT_EXPIRY_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(group_id: Uuid) -> Self {
        let now = Utc::now();
        let expiry = now + Duration::days(JWT_EXPIRY_DAYS);

        Self {
            sub: group_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
        }
    }

    pub fn group_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| AppError::InvalidInput("Invalid group ID in JWT".to_string()))
    }
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::PasswordHashError(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<()> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| AppError::PasswordHashError(e.to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)
}

pub fn create_jwt(group_id: Uuid, secret: &str) -> Result<String> {
    let claims = Claims::new(group_id);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::from)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(AppError::from)
}
