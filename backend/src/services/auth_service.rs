//! Authentication service for user management and JWT token handling

use crate::config::Config;
use crate::error::AppError;
use crate::error::AppResult;
use crate::models::{
    Claims, LoginRequest, LoginResponse, RegisterRequest, User, UserInfo, UserSession,
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub struct AuthService;

impl AuthService {
    /// Register a new user
    pub async fn register(
        pool: &PgPool,
        config: &Config,
        input: RegisterRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> AppResult<LoginResponse> {
        // Validate input
        input.validate()?;

        // Check if user already exists
        if let Some(_) = User::find_by_email(pool, &input.email).await? {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Hash password using Argon2id
        let password_hash = Self::hash_password(&input.password)?;

        // Create user
        let user = User::create(
            pool,
            &input.email,
            &password_hash,
            input.display_name.as_deref(),
        )
        .await?;

        // Generate tokens
        let (access_token, access_expires_at) =
            Self::generate_access_token(user.id, &user.email, config)?;
        let (refresh_token, refresh_token_hash, refresh_expires_at) =
            Self::generate_refresh_token(config)?;

        // Create session
        UserSession::create(
            pool,
            user.id,
            &refresh_token_hash,
            refresh_expires_at,
            user_agent.as_deref(),
            ip_address.as_deref(),
        )
        .await?;

        // Update last login
        User::update_last_login(pool, user.id).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user.to_user_info(),
            access_token_expires_at: access_expires_at,
            refresh_token_expires_at: refresh_expires_at,
        })
    }

    /// Login an existing user
    pub async fn login(
        pool: &PgPool,
        config: &Config,
        input: LoginRequest,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> AppResult<LoginResponse> {
        // Validate input
        input.validate()?;

        // Find user by email
        let user = User::find_by_email(pool, &input.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".to_string()));
        }

        // Verify password
        Self::verify_password(&input.password, &user.password_hash)?;

        // Generate tokens
        let (access_token, access_expires_at) =
            Self::generate_access_token(user.id, &user.email, config)?;
        let (refresh_token, refresh_token_hash, refresh_expires_at) =
            Self::generate_refresh_token(config)?;

        // Create session
        UserSession::create(
            pool,
            user.id,
            &refresh_token_hash,
            refresh_expires_at,
            user_agent.as_deref(),
            ip_address.as_deref(),
        )
        .await?;

        // Update last login
        User::update_last_login(pool, user.id).await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            user: user.to_user_info(),
            access_token_expires_at: access_expires_at,
            refresh_token_expires_at: refresh_expires_at,
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(
        pool: &PgPool,
        config: &Config,
        refresh_token: &str,
    ) -> AppResult<LoginResponse> {
        // Hash the refresh token to compare with stored hash
        let refresh_token_hash = Self::hash_token(refresh_token);

        // Find session by token hash
        let session = UserSession::find_by_token_hash(pool, &refresh_token_hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".to_string()))?;

        // Check if session is expired
        if session.expires_at < Utc::now() {
            return Err(AppError::Unauthorized("Refresh token expired".to_string()));
        }

        // Get user
        let user = User::find_by_id(pool, session.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".to_string()));
        }

        // Generate new access token
        let (access_token, access_expires_at) =
            Self::generate_access_token(user.id, &user.email, config)?;

        Ok(LoginResponse {
            access_token,
            refresh_token: refresh_token.to_string(), // Return same refresh token
            user: user.to_user_info(),
            access_token_expires_at: access_expires_at,
            refresh_token_expires_at: session.expires_at,
        })
    }

    /// Logout user (revoke refresh token)
    pub async fn logout(pool: &PgPool, refresh_token: &str) -> AppResult<()> {
        let refresh_token_hash = Self::hash_token(refresh_token);
        UserSession::revoke(pool, &refresh_token_hash).await?;
        Ok(())
    }

    /// Get current user from access token
    pub async fn get_current_user(pool: &PgPool, user_id: Uuid) -> AppResult<UserInfo> {
        let user = User::find_by_id(pool, user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("Account is disabled".to_string()));
        }

        Ok(user.to_user_info())
    }

    /// Verify JWT access token
    pub fn verify_access_token(token: &str, config: &Config) -> AppResult<Claims> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            log::debug!("JWT verification failed: {:?}", e);
            AppError::Unauthorized("Invalid token".to_string())
        })?;

        // Verify token type
        if token_data.claims.token_type != "access" {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        Ok(token_data.claims)
    }

    /// Hash password using Argon2id
    fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                log::error!("Password hashing failed: {:?}", e);
                AppError::InternalError("Password hashing failed".to_string())
            })?
            .to_string();

        Ok(password_hash)
    }

    /// Verify password against hash
    fn verify_password(password: &str, hash: &str) -> AppResult<()> {
        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            log::error!("Password hash parsing failed: {:?}", e);
            AppError::InternalError("Password verification failed".to_string())
        })?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

        Ok(())
    }

    /// Generate JWT access token
    fn generate_access_token(
        user_id: Uuid,
        email: &str,
        config: &Config,
    ) -> AppResult<(String, chrono::DateTime<Utc>)> {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(config.jwt_access_token_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|e| {
            log::error!("JWT encoding failed: {:?}", e);
            AppError::InternalError("Token generation failed".to_string())
        })?;

        Ok((token, expires_at))
    }

    /// Generate random refresh token
    fn generate_refresh_token(
        config: &Config,
    ) -> AppResult<(String, String, chrono::DateTime<Utc>)> {
        // Generate 32 random bytes
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut random_bytes = [0u8; 32];
        rng.fill_bytes(&mut random_bytes);
        let refresh_token = format!("rt_{}", hex::encode(&random_bytes));

        // Hash the token for storage
        let refresh_token_hash = Self::hash_token(&refresh_token);

        // Calculate expiration
        let expires_at = Utc::now() + Duration::seconds(config.jwt_refresh_token_expiry);

        Ok((refresh_token, refresh_token_hash, expires_at))
    }

    /// Hash token using SHA-256
    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
