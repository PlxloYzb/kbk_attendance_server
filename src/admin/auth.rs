use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::admin::models::*;
use crate::models::ApiResponse;

lazy_static! {
    static ref ADMIN_SESSIONS: Mutex<HashMap<String, AdminSession>> = Mutex::new(HashMap::new());
}

pub async fn admin_login(
    pool: web::Data<PgPool>,
    req: web::Json<AdminLoginRequest>,
) -> HttpResponse {
    match sqlx::query_as::<_, AdminUser>(
        "SELECT * FROM admin_user WHERE username = $1 AND password = $2"
    )
    .bind(&req.username)
    .bind(&req.password)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(user)) => {
            let token = Uuid::new_v4().to_string();
            let session = AdminSession {
                user_id: user.id,
                username: user.username.clone(),
                role: user.role.clone(),
                department: user.department,
            };
            
            let mut sessions = ADMIN_SESSIONS.lock().unwrap();
            sessions.insert(token.clone(), session);
            
            let user_info = AdminUserInfo {
                id: user.id,
                username: user.username,
                role: user.role,
                department: user.department,
            };
            
            let response = AdminLoginResponse {
                token,
                user: user_info,
            };
            
            HttpResponse::Ok().json(ApiResponse::success(response, "Login successful"))
        }
        Ok(None) => {
            HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"))
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error"))
        }
    }
}

pub async fn get_admin_info(req: HttpRequest) -> HttpResponse {
    if let Some(session) = get_session_from_request(&req) {
        let user_info = AdminUserInfo {
            id: session.user_id,
            username: session.username,
            role: session.role,
            department: session.department,
        };
        HttpResponse::Ok().json(ApiResponse::success(user_info, "User info retrieved"))
    } else {
        HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid or expired token"))
    }
}

pub fn verify_admin_token(token: &str) -> Option<AdminSession> {
    let sessions = ADMIN_SESSIONS.lock().unwrap();
    sessions.get(token).cloned()
}

pub fn get_session_from_request(req: &HttpRequest) -> Option<AdminSession> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                return verify_admin_token(token);
            }
        }
    }
    None
}

// Helper function to check authentication in handlers
pub fn require_admin_auth(req: &HttpRequest) -> Result<AdminSession, HttpResponse> {
    if let Some(session) = get_session_from_request(req) {
        Ok(session)
    } else {
        Err(HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Authentication required")))
    }
}