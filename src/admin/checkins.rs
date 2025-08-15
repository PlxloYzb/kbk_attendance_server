use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{CreateCheckinRequest, UpdateCheckinRequest};
use crate::models::{ApiResponse, Checkin};

pub async fn get_checkins(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<CheckinQuery>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    let mut sql = "SELECT * FROM checkins".to_string();
    let mut conditions = Vec::new();
    
    if let Some(user_id) = &query.user_id {
        conditions.push(format!("user_id = '{}'", user_id));
    }
    
    if let Some(action) = &query.action {
        conditions.push(format!("action = '{}'", action));
    }
    
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    
    sql.push_str(" ORDER BY created_at DESC");
    
    if let Some(limit) = query.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    } else {
        sql.push_str(" LIMIT 100"); // Default limit
    }

    match sqlx::query_as::<_, Checkin>(&sql)
        .fetch_all(pool.as_ref())
        .await
    {
        Ok(checkins) => HttpResponse::Ok().json(ApiResponse::success(checkins, "Checkins retrieved")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve checkins")),
    }
}

pub async fn create_checkin(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    checkin_req: web::Json<CreateCheckinRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    // Start a transaction to ensure atomicity
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Transaction failed")),
    };

    // First, insert the checkin record
    let checkin_result = sqlx::query_as::<_, Checkin>(
        r#"
        INSERT INTO checkins (user_id, action, created_at, latitude, longitude, is_synced)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(&checkin_req.user_id)
    .bind(&checkin_req.action)
    .bind(checkin_req.created_at)
    .bind(checkin_req.latitude)
    .bind(checkin_req.longitude)
    .bind(checkin_req.is_synced)
    .fetch_one(&mut *tx)
    .await;

    let checkin = match checkin_result {
        Ok(checkin) => checkin,
        Err(_) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create checkin"));
        }
    };

    // Now process the checkin into attendance_sessions using the same logic as sync endpoint
    let date = checkin_req.created_at.date_naive();
    
    // Get the current max session number for this user and date
    let mut session_number = 
        match sqlx::query_scalar::<_, i32>(
            "SELECT COALESCE(MAX(session_number), 0) FROM attendance_sessions 
             WHERE user_id = $1 AND date = $2"
        )
        .bind(&checkin_req.user_id)
        .bind(date)
        .fetch_one(&mut *tx)
        .await {
            Ok(num) => num,
            Err(_) => 0,
        };

    // Process the checkin action
    if checkin_req.action == "IN" {
        // Check if there's an incomplete session (no checkout)
        let has_incomplete_session = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM attendance_sessions 
             WHERE user_id = $1 AND date = $2 AND checkout_time IS NULL)"
        )
        .bind(&checkin_req.user_id)
        .bind(date)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(false);

        if !has_incomplete_session {
            // Start new session
            session_number += 1;
            let result = sqlx::query(
                "INSERT INTO attendance_sessions 
                 (user_id, date, session_number, checkin_time, checkin_latitude, checkin_longitude) 
                 VALUES ($1, $2, $3, $4, $5, $6)
                 ON CONFLICT (user_id, date, session_number) 
                 DO UPDATE SET 
                    checkin_time = EXCLUDED.checkin_time,
                    checkin_latitude = EXCLUDED.checkin_latitude,
                    checkin_longitude = EXCLUDED.checkin_longitude"
            )
            .bind(&checkin_req.user_id)
            .bind(date)
            .bind(session_number)
            .bind(checkin_req.created_at)
            .bind(checkin_req.latitude)
            .bind(checkin_req.longitude)
            .execute(&mut *tx)
            .await;

            if result.is_err() {
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create session"));
            }
        }
    } else if checkin_req.action == "OUT" {
        // Try to close an existing incomplete session (including previous day for midnight crossings)
        let update_result = sqlx::query(
            "UPDATE attendance_sessions 
             SET checkout_time = $1, checkout_latitude = $2, checkout_longitude = $3
             WHERE user_id = $4 
                AND date IN ($5, $5 - INTERVAL '1 day')
                AND checkout_time IS NULL
                AND checkin_time < $1 + INTERVAL '16 hours'  -- Max session duration
             ORDER BY checkin_time DESC
             LIMIT 1"
        )
        .bind(checkin_req.created_at)
        .bind(checkin_req.latitude)
        .bind(checkin_req.longitude)
        .bind(&checkin_req.user_id)
        .bind(date)
        .execute(&mut *tx)
        .await;

        match update_result {
            Ok(result) if result.rows_affected() == 0 => {
                // No incomplete session found, create a checkout-only session
                session_number += 1;
                let result = sqlx::query(
                    "INSERT INTO attendance_sessions 
                     (user_id, date, session_number, checkin_time, checkout_time, checkout_latitude, checkout_longitude) 
                     VALUES ($1, $2, $3, $4, $5, $6, $7)
                     ON CONFLICT (user_id, date, session_number) DO NOTHING"
                )
                .bind(&checkin_req.user_id)
                .bind(date)
                .bind(session_number)
                .bind(checkin_req.created_at) // Use checkout time as checkin for incomplete session
                .bind(checkin_req.created_at)
                .bind(checkin_req.latitude)
                .bind(checkin_req.longitude)
                .execute(&mut *tx)
                .await;

                if result.is_err() {
                    let _ = tx.rollback().await;
                    return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create incomplete session"));
                }
            }
            Err(_) => {
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update session"));
            }
            _ => {} // Successfully updated existing session
        }
    }

    // Commit the transaction - attendance_summary will be automatically updated by triggers
    match tx.commit().await {
        Ok(_) => HttpResponse::Created().json(ApiResponse::success(checkin, "Checkin created and processed into sessions")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to commit transaction")),
    }
}

// Note: Updating checkins is complex as it may require reprocessing attendance sessions.
// For now, this function only updates the checkin record itself.
// If you need to update attendance data, consider deleting and recreating the checkin,
// or manually run the populate_sessions_from_checkins.sql script.
pub async fn update_checkin(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    checkin_req: web::Json<UpdateCheckinRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    let id = path.into_inner();
    
    match sqlx::query_as::<_, Checkin>(
        r#"
        UPDATE checkins 
        SET user_id = $1, action = $2, created_at = $3, latitude = $4, longitude = $5, is_synced = $6
        WHERE id = $7
        RETURNING *
        "#
    )
    .bind(&checkin_req.user_id)
    .bind(&checkin_req.action)
    .bind(checkin_req.created_at)
    .bind(checkin_req.latitude)
    .bind(checkin_req.longitude)
    .bind(checkin_req.is_synced)
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(checkin)) => HttpResponse::Ok().json(ApiResponse::success(checkin, "Checkin updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("Checkin not found")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update checkin")),
    }
}

pub async fn delete_checkin(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    let id = path.into_inner();
    
    match sqlx::query("DELETE FROM checkins WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Checkin deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Checkin not found"))
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete checkin")),
    }
}

#[derive(serde::Deserialize)]
pub struct CheckinQuery {
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub limit: Option<i64>,
}