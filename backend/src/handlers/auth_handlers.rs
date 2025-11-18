use actix_web::{HttpRequest, HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth_middleware::auth::AuthenticatedUser;
use crate::config::Config;
use crate::error::AppError;
use crate::models::{LoginRequest, RegisterRequest, UserInfo};
use crate::services::AuthService;

// Request DTOs (beyond what's in models)
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
}

/// Extract user agent from request
fn get_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract IP address from request
fn get_ip_address(req: &HttpRequest) -> Option<String> {
    req.peer_addr().map(|addr| addr.ip().to_string())
}

/// POST /api/auth/register
/// Register a new user
pub async fn register(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req_data: web::Json<RegisterRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_agent = get_user_agent(&req);
    let ip_address = get_ip_address(&req);

    let response = AuthService::register(
        pool.get_ref(),
        config.get_ref(),
        req_data.into_inner(),
        user_agent,
        ip_address,
    )
    .await?;

    let auth_response = AuthResponse {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        user: response.user,
        access_token_expires_at: response.access_token_expires_at,
        refresh_token_expires_at: response.refresh_token_expires_at,
    };

    Ok(HttpResponse::Created().json(auth_response))
}

/// POST /api/auth/login
/// Login a user with email and password
pub async fn login(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req_data: web::Json<LoginRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_agent = get_user_agent(&req);
    let ip_address = get_ip_address(&req);

    let response = AuthService::login(
        pool.get_ref(),
        config.get_ref(),
        req_data.into_inner(),
        user_agent,
        ip_address,
    )
    .await?;

    let auth_response = AuthResponse {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        user: response.user,
        access_token_expires_at: response.access_token_expires_at,
        refresh_token_expires_at: response.refresh_token_expires_at,
    };

    Ok(HttpResponse::Ok().json(auth_response))
}

/// POST /api/auth/refresh
/// Refresh access token using refresh token
pub async fn refresh(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    req_data: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let response =
        AuthService::refresh_token(pool.get_ref(), config.get_ref(), &req_data.refresh_token)
            .await?;

    let auth_response = AuthResponse {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        user: response.user,
        access_token_expires_at: response.access_token_expires_at,
        refresh_token_expires_at: response.refresh_token_expires_at,
    };

    Ok(HttpResponse::Ok().json(auth_response))
}

/// POST /api/auth/logout
/// Logout user by invalidating refresh token
pub async fn logout(
    pool: web::Data<PgPool>,
    req_data: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::logout(pool.get_ref(), &req_data.refresh_token).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

/// GET /api/auth/me
/// Get current user information (requires authentication)
pub async fn get_current_user(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, AppError> {
    let user_info = AuthService::get_current_user(pool.get_ref(), user.user_id).await?;

    Ok(HttpResponse::Ok().json(user_info))
}
