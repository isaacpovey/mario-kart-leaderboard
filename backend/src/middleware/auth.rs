use crate::auth::verify_jwt;
use crate::config::Config;
use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthMiddleware;

/// Pure function to extract and validate group_id from authorization header
fn extract_group_id(auth_header: Option<&str>, jwt_secret: &str) -> Option<Uuid> {
    auth_header
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .and_then(|token| verify_jwt(token, jwt_secret).ok())
        .and_then(|claims| claims.group_id().ok())
}

pub async fn auth_middleware(
    config: axum::extract::State<Config>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header value
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    // Extract and validate group_id using pure function
    let group_id = extract_group_id(auth_header, &config.jwt_secret);

    // Transform request by adding group_id to extensions
    // Note: This mutation is unavoidable with Axum's design, but isolated
    let (mut parts, body) = req.into_parts();
    parts.extensions.insert(group_id);
    let req = Request::from_parts(parts, body);

    Ok(next.run(req).await)
}
