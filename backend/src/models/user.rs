//! User model and authentication data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

/// User model representing a registered user
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// User session model for refresh token management
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// Registration request input
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(max = 100, message = "Display name must be at most 100 characters"))]
    pub display_name: Option<String>,
}

/// Login request input
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

/// Login/Register response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
    pub access_token_expires_at: DateTime<Utc>,
    pub refresh_token_expires_at: DateTime<Utc>,
}

/// User information (safe for client)
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // Subject (user ID)
    pub email: String, // User email
    pub exp: i64,      // Expiration time
    pub iat: i64,      // Issued at
    #[serde(rename = "type")]
    pub token_type: String, // Token type: "access" or "refresh"
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl User {
    /// Create a new user
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, display_name)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, display_name, created_at, updated_at, last_login_at, is_active
            "#,
            email,
            password_hash,
            display_name
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, display_name, created_at, updated_at, last_login_at, is_active
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, password_hash, display_name, created_at, updated_at, last_login_at, is_active
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE users
            SET last_login_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Convert to UserInfo (safe for client)
    pub fn to_user_info(&self) -> UserInfo {
        UserInfo {
            id: self.id,
            email: self.email.clone(),
            display_name: self.display_name.clone(),
            created_at: self.created_at,
            last_login_at: self.last_login_at,
        }
    }
}

impl UserSession {
    /// Create a new session
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        refresh_token_hash: &str,
        expires_at: DateTime<Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let session = sqlx::query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at, user_agent, ip_address)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, refresh_token_hash, expires_at, created_at, is_active, user_agent, ip_address
            "#,
            user_id,
            refresh_token_hash,
            expires_at,
            user_agent,
            ip_address
        )
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    /// Find session by refresh token hash
    pub async fn find_by_token_hash(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let session = sqlx::query_as!(
            UserSession,
            r#"
            SELECT id, user_id, refresh_token_hash, expires_at, created_at, is_active, user_agent, ip_address
            FROM user_sessions
            WHERE refresh_token_hash = $1 AND is_active = TRUE
            "#,
            token_hash
        )
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    /// Revoke a session (logout)
    pub async fn revoke(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE user_sessions
            SET is_active = FALSE
            WHERE refresh_token_hash = $1
            "#,
            token_hash
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE user_sessions
            SET is_active = FALSE
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
