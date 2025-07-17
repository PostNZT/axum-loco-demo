use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("User not found")]
    UserNotFound,
    #[error("Email already exists")]
    EmailAlreadyExists,
    #[error("Password hashing failed")]
    PasswordHashingFailed,
    #[error("JWT error: {0}")]
    JwtError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: String,
    pub name: String,
    pub exp: i64, // Expiration time
    pub iat: i64, // Issued at
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, name: String, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);
        
        Self {
            sub: user_id.to_string(),
            email,
            name,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| AuthError::PasswordHashingFailed)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        bcrypt::verify(password, hash)
            .map_err(|_| AuthError::InvalidCredentials)
    }

    pub fn generate_token(&self, claims: &Claims) -> Result<String, AuthError> {
        use jsonwebtoken::{encode, Header, EncodingKey};
        
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| AuthError::JwtError(e.to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| AuthError::JwtError(e.to_string()))?;

        let claims = token_data.claims;
        
        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(claims)
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            token_expiry_hours: 24,
            refresh_token_expiry_days: 30,
        }
    }
}

// Middleware helper for extracting user from token
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl AuthenticatedUser {
    pub fn from_claims(claims: Claims) -> Result<Self, AuthError> {
        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;
        
        Ok(Self {
            id,
            email: claims.email,
            name: claims.name,
        })
    }
}

// Password validation utilities
pub struct PasswordValidator;

impl PasswordValidator {
    pub fn validate(password: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if password.len() < 8 {
            errors.push("Password must be at least 8 characters long".to_string());
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }

        if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            errors.push("Password must contain at least one special character".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Rate limiting for authentication attempts
#[derive(Debug, Clone)]
pub struct RateLimiter {
    #[allow(dead_code)]
    max_attempts: u32,
    #[allow(dead_code)]
    window_minutes: u32,
}

impl RateLimiter {
    pub fn new(max_attempts: u32, window_minutes: u32) -> Self {
        Self {
            max_attempts,
            window_minutes,
        }
    }

    // In a real implementation, this would use Redis or similar
    pub fn check_rate_limit(&self, _identifier: &str) -> bool {
        // Simplified implementation - always allow for demo
        true
    }

    pub fn record_attempt(&self, _identifier: &str) {
        // Record the attempt in storage
    }
}
