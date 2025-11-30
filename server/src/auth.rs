use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::database::{get_player_by_username, JwtTokenDb};

// JWT secret key - in production, this should be loaded from environment variables
const JWT_SECRET: &str = "your-secret-key-change-in-production";
const JWT_EXPIRY_HOURS: i64 = 24; // Token expires after 24 hours

// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // Subject (user ID)
    pub username: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
}

// Authentication error type
#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    InvalidToken,
    DatabaseError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid username or password"),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

// Generate a JWT token for a user
pub fn generate_token(player_id: Uuid, username: &str) -> Result<String, AuthError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(JWT_EXPIRY_HOURS)).timestamp();

    let claims = Claims {
        sub: player_id,
        username: username.to_string(),
        exp,
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .map_err(|_| AuthError::InvalidToken)
}

// Validate and decode a JWT token
pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &validation,
    )
    .map_err(|e| {
        if e.to_string().contains("expired") {
            AuthError::TokenExpired
        } else {
            AuthError::InvalidToken
        }
    })?;

    Ok(token_data.claims)
}

// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(password, hash).map_err(|_| AuthError::InvalidCredentials)
}

// Hash a password
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|_| AuthError::DatabaseError("Failed to hash password".to_string()))
}

// Authenticate user with username and password
pub async fn authenticate_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<(Uuid, String), AuthError> {
    // Get player from database
    let player = get_player_by_username(pool, username)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    // Check if password_hash exists
    let password_hash = player
        .password_hash
        .as_ref()
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    if !verify_password(password, password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    // Generate token
    let token = generate_token(player.id, &player.username)?;

    Ok((player.id, token))
}

// Store JWT token in database
pub async fn store_token(
    pool: &PgPool,
    player_id: Uuid,
    token: &str,
) -> Result<JwtTokenDb, AuthError> {
    let expires_at = Utc::now() + Duration::hours(JWT_EXPIRY_HOURS);

    let jwt_token = sqlx::query_as::<_, JwtTokenDb>(
        r#"
        INSERT INTO jwt_token (player_id, token, expires_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(player_id)
    .bind(token)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(jwt_token)
}

// Validate token exists in database and is not expired
pub async fn validate_token_in_db(pool: &PgPool, token: &str) -> Result<JwtTokenDb, AuthError> {
    let jwt_token = sqlx::query_as::<_, JwtTokenDb>(
        r#"
        SELECT * FROM jwt_token
        WHERE token = $1 AND expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::TokenExpired)?;

    Ok(jwt_token)
}

// Delete expired tokens (cleanup function)
pub async fn cleanup_expired_tokens(pool: &PgPool) -> Result<u64, AuthError> {
    let result = sqlx::query(
        r#"
        DELETE FROM jwt_token
        WHERE expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected())
}

// Delete a specific token (logout)
pub async fn delete_token(pool: &PgPool, token: &str) -> Result<bool, AuthError> {
    let result = sqlx::query("DELETE FROM jwt_token WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected() > 0)
}
