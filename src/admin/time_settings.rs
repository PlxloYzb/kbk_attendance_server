use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use chrono::NaiveTime;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{
    BatchUpdateTimeSettingsRequest, UserWithTimeSetting
};
use crate::models::ApiResponse;

pub async fn get_users_with_time_settings(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Only admin can access time settings
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
    }

    let department_filter = query.get("department")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i32>().ok());

    let result = if let Some(dept) = department_filter {
        sqlx::query_as::<_, (String, Option<String>, i32, Option<String>, Option<NaiveTime>, Option<NaiveTime>)>(r#"
            SELECT 
                ui.user_id,
                ui.user_name,
                ui.department,
                ui.department_name,
                uts.on_duty_time,
                uts.off_duty_time
            FROM user_info ui
            LEFT JOIN user_time_settings uts ON ui.user_id = uts.user_id
            WHERE ui.department = $1
            ORDER BY ui.department, ui.user_id
        "#)
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, (String, Option<String>, i32, Option<String>, Option<NaiveTime>, Option<NaiveTime>)>(r#"
            SELECT 
                ui.user_id,
                ui.user_name,
                ui.department,
                ui.department_name,
                uts.on_duty_time,
                uts.off_duty_time
            FROM user_info ui
            LEFT JOIN user_time_settings uts ON ui.user_id = uts.user_id
            ORDER BY ui.department, ui.user_id
        "#)
        .fetch_all(pool.as_ref())
        .await
    };

    match result {
        Ok(rows) => {
            let users: Vec<UserWithTimeSetting> = rows.into_iter().map(|(user_id, user_name, department, department_name, on_duty_time, off_duty_time)| {
                UserWithTimeSetting {
                    user_id,
                    user_name,
                    department,
                    department_name,
                    on_duty_time,
                    off_duty_time,
                }
            }).collect();
            
            HttpResponse::Ok().json(ApiResponse::success(users, "Users with time settings retrieved"))
        }
        Err(e) => {
            log::error!("Failed to retrieve users with time settings: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve users"))
        }
    }
}

pub async fn batch_update_time_settings(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<BatchUpdateTimeSettingsRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Only admin can modify time settings
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
    }

    let mut transaction = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error"));
        }
    };

    let mut updated_count = 0;

    for setting in &body.settings {
        // Parse time strings
        let on_duty_time = match NaiveTime::parse_from_str(&setting.on_duty_time, "%H:%M:%S") {
            Ok(time) => time,
            Err(_) => {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&format!("Invalid on_duty_time format for user {}: {}", setting.user_id, setting.on_duty_time)));
            }
        };

        let off_duty_time = match NaiveTime::parse_from_str(&setting.off_duty_time, "%H:%M:%S") {
            Ok(time) => time,
            Err(_) => {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&format!("Invalid off_duty_time format for user {}: {}", setting.user_id, setting.off_duty_time)));
            }
        };

        // UPSERT time setting
        let upsert_result = sqlx::query(r#"
            INSERT INTO user_time_settings (user_id, on_duty_time, off_duty_time)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                on_duty_time = EXCLUDED.on_duty_time,
                off_duty_time = EXCLUDED.off_duty_time,
                updated_at = NOW()
        "#)
        .bind(&setting.user_id)
        .bind(on_duty_time)
        .bind(off_duty_time)
        .execute(&mut *transaction)
        .await;

        match upsert_result {
            Ok(_) => updated_count += 1,
            Err(e) => {
                log::error!("Failed to upsert time setting for user {}: {:?}", setting.user_id, e);
                if let Err(rollback_err) = transaction.rollback().await {
                    log::error!("Failed to rollback transaction: {:?}", rollback_err);
                }
                return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&format!("Failed to update time setting for user {}", setting.user_id)));
            }
        }
    }

    match transaction.commit().await {
        Ok(_) => {
            HttpResponse::Ok().json(ApiResponse::success(
                updated_count, 
                &format!("Successfully updated {} time settings", updated_count)
            ))
        }
        Err(e) => {
            log::error!("Failed to commit transaction: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to save time settings"))
        }
    }
}

pub async fn delete_time_setting(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Only admin can delete time settings
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
    }

    let user_id = path.into_inner();

    let result = sqlx::query("DELETE FROM user_time_settings WHERE user_id = $1")
        .bind(&user_id)
        .execute(pool.as_ref())
        .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::success((), "Time setting deleted successfully"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Time setting not found"))
            }
        }
        Err(e) => {
            log::error!("Failed to delete time setting: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete time setting"))
        }
    }
}
