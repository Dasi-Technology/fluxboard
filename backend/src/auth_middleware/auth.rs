use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::future::{Ready, ready};
use uuid::Uuid;

use crate::config::Config;
use crate::error::AppError;
use crate::services::AuthService;

/// Authenticated user information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

/// Extract user from request extensions (set by middleware)
impl actix_web::FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let user = req.extensions().get::<AuthenticatedUser>().cloned();

        match user {
            Some(user) => ready(Ok(user)),
            None => ready(Err(AppError::Unauthorized(
                "Authentication required".to_string(),
            )
            .into())),
        }
    }
}

/// Optional authenticated user (doesn't fail if not present)
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AuthenticatedUser>);

impl actix_web::FromRequest for OptionalUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let user = req.extensions().get::<AuthenticatedUser>().cloned();
        ready(Ok(OptionalUser(user)))
    }
}

/// Required authentication middleware
/// Returns 401 if no valid token is present
pub struct RequireAuth {
    config: Config,
}

impl RequireAuth {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service,
            config: self.config.clone(),
        }))
    }
}

pub struct RequireAuthMiddleware<S> {
    service: S,
    config: Config,
}

impl<S, B> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = self.config.clone();

        // Extract token from Authorization header
        let auth_header = req.headers().get("Authorization");

        let token = match auth_header {
            Some(header) => {
                let auth_str = match header.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return Box::pin(async move {
                            Err(
                                AppError::Unauthorized("Invalid authorization header".to_string())
                                    .into(),
                            )
                        });
                    }
                };

                if !auth_str.starts_with("Bearer ") {
                    return Box::pin(async move {
                        Err(
                            AppError::Unauthorized("Invalid authorization format".to_string())
                                .into(),
                        )
                    });
                }

                auth_str[7..].to_string()
            }
            None => {
                return Box::pin(async move {
                    Err(AppError::Unauthorized("Missing authorization header".to_string()).into())
                });
            }
        };

        // Validate token
        let claims = match AuthService::verify_access_token(&token, &config) {
            Ok(claims) => claims,
            Err(_) => {
                return Box::pin(async move {
                    Err(AppError::Unauthorized("Invalid or expired token".to_string()).into())
                });
            }
        };

        // Parse user_id from claims
        let user_id = match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => {
                return Box::pin(async move {
                    Err(AppError::Unauthorized("Invalid user ID in token".to_string()).into())
                });
            }
        };

        // Store authenticated user in request extensions
        req.extensions_mut().insert(AuthenticatedUser { user_id });

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

/// Optional authentication middleware
/// Validates token if present, but doesn't fail if missing
pub struct OptionalAuth {
    config: Config,
}

impl OptionalAuth {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for OptionalAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OptionalAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OptionalAuthMiddleware {
            service,
            config: self.config.clone(),
        }))
    }
}

pub struct OptionalAuthMiddleware<S> {
    service: S,
    config: Config,
}

impl<S, B> Service<ServiceRequest> for OptionalAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = self.config.clone();

        // Extract token from Authorization header
        let auth_header = req.headers().get("Authorization");

        if let Some(header) = auth_header {
            if let Ok(auth_str) = header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];

                    // Validate token
                    if let Ok(claims) = AuthService::verify_access_token(token, &config) {
                        // Parse user_id from claims
                        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                            // Store authenticated user in request extensions
                            req.extensions_mut().insert(AuthenticatedUser { user_id });
                        }
                    }
                }
            }
        }

        // Continue regardless of authentication status
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
