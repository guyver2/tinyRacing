use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::TypedHeader;
use headers::authorization::Bearer;
use headers::Authorization;

use crate::api::ApiError;
use crate::auth::{validate_token, validate_token_in_db, AuthError, Claims};
use sqlx::PgPool;

// Extractor for authenticated user claims
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

// Middleware to validate JWT token
pub async fn auth_middleware(
    State(pool): State<PgPool>,
    TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = auth_header.token();

    // Validate token structure
    let claims = validate_token(token).map_err(|e| match e {
        AuthError::TokenExpired => ApiError::Unauthorized("Token has expired".to_string()),
        AuthError::InvalidToken => ApiError::Unauthorized("Invalid token".to_string()),
        _ => ApiError::Unauthorized("Authentication failed".to_string()),
    })?;

    // Validate token exists in database and is not expired
    validate_token_in_db(&pool, token)
        .await
        .map_err(|e| match e {
            AuthError::TokenExpired => ApiError::Unauthorized("Token has expired".to_string()),
            AuthError::DatabaseError(msg) => ApiError::InternalError(msg),
            _ => ApiError::Unauthorized("Invalid token".to_string()),
        })?;

    // Add claims to request extensions for use in handlers
    request
        .extensions_mut()
        .insert(AuthenticatedUser { claims });

    Ok(next.run(request).await)
}
