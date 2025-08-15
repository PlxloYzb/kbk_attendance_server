use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use std::sync::Arc;

use crate::admin::auth::require_admin_auth;
use crate::models::ApiResponse;
use crate::sync::SyncService;

#[derive(serde::Serialize)]
struct SyncResponse {
    synced_count: usize,
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Manual sync endpoint for admin to trigger user time settings sync
pub async fn manual_sync_time_settings(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Only admin can trigger manual sync
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
    }

    // Create sync service instance
    let sync_service = SyncService::new(Arc::new(pool.get_ref().clone()));
    
    match sync_service.manual_sync().await {
        Ok((count, message)) => {
            let response = SyncResponse {
                synced_count: count,
                message,
                timestamp: chrono::Utc::now(),
            };
            HttpResponse::Ok().json(ApiResponse::success(response, "Sync completed successfully"))
        }
        Err(error_message) => {
            log::error!("Manual sync failed: {}", error_message);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&error_message))
        }
    }
}

/// Get sync status information
pub async fn get_sync_status(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
    }

    match get_sync_status_info(pool.as_ref()).await {
        Ok(status) => HttpResponse::Ok().json(ApiResponse::success(status, "Sync status retrieved")),
        Err(e) => {
            log::error!("Failed to get sync status: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to get sync status"))
        }
    }
}

#[derive(serde::Serialize)]
struct SyncStatus {
    total_users: i64,
    users_with_time_settings: i64,
    is_synced: bool,
    missing_count: i64,
}

async fn get_sync_status_info(pool: &PgPool) -> Result<SyncStatus, sqlx::Error> {
    let result: (i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM user_info) as total_users,
            (SELECT COUNT(*) FROM user_time_settings) as users_with_settings
        "#
    )
    .fetch_one(pool)
    .await?;

    let total_users = result.0;
    let users_with_settings = result.1;
    let missing_count = total_users - users_with_settings;
    let is_synced = missing_count == 0;

    Ok(SyncStatus {
        total_users,
        users_with_time_settings: users_with_settings,
        is_synced,
        missing_count,
    })
}
