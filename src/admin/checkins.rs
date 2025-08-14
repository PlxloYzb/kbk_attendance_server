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

    match sqlx::query_as::<_, Checkin>(
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
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(checkin) => HttpResponse::Created().json(ApiResponse::success(checkin, "Checkin created")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create checkin")),
    }
}

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