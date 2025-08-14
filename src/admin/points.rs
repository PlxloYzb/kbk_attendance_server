use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{CreatePointRequest, UpdatePointRequest};
use crate::models::{ApiResponse, CheckinPoint, CheckoutPoint};

pub async fn get_checkin_points(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    match sqlx::query_as::<_, CheckinPoint>("SELECT * FROM checkin_points ORDER BY id")
        .fetch_all(pool.as_ref())
        .await
    {
        Ok(points) => HttpResponse::Ok().json(ApiResponse::success(points, "Checkin points retrieved")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve checkin points")),
    }
}

pub async fn create_checkin_point(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    point_req: web::Json<CreatePointRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    match sqlx::query_as::<_, CheckinPoint>(
        r#"
        INSERT INTO checkin_points (latitude, longitude, radius, location_name, allowed_department)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(point_req.latitude)
    .bind(point_req.longitude)
    .bind(point_req.radius)
    .bind(&point_req.location_name)
    .bind(&point_req.allowed_department)
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(point) => HttpResponse::Created().json(ApiResponse::success(point, "Checkin point created")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create checkin point")),
    }
}

pub async fn update_checkin_point(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    point_req: web::Json<UpdatePointRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    let id = path.into_inner();
    
    match sqlx::query_as::<_, CheckinPoint>(
        r#"
        UPDATE checkin_points 
        SET latitude = $1, longitude = $2, radius = $3, location_name = $4, allowed_department = $5
        WHERE id = $6
        RETURNING *
        "#
    )
    .bind(point_req.latitude)
    .bind(point_req.longitude)
    .bind(point_req.radius)
    .bind(&point_req.location_name)
    .bind(&point_req.allowed_department)
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(point)) => HttpResponse::Ok().json(ApiResponse::success(point, "Checkin point updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("Checkin point not found")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update checkin point")),
    }
}

pub async fn delete_checkin_point(
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
    
    match sqlx::query("DELETE FROM checkin_points WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Checkin point deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Checkin point not found"))
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete checkin point")),
    }
}

pub async fn get_checkout_points(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    match sqlx::query_as::<_, CheckoutPoint>("SELECT * FROM checkout_points ORDER BY id")
        .fetch_all(pool.as_ref())
        .await
    {
        Ok(points) => HttpResponse::Ok().json(ApiResponse::success(points, "Checkout points retrieved")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve checkout points")),
    }
}

pub async fn create_checkout_point(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    point_req: web::Json<CreatePointRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    match sqlx::query_as::<_, CheckoutPoint>(
        r#"
        INSERT INTO checkout_points (latitude, longitude, radius, location_name, allowed_department)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(point_req.latitude)
    .bind(point_req.longitude)
    .bind(point_req.radius)
    .bind(&point_req.location_name)
    .bind(&point_req.allowed_department)
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(point) => HttpResponse::Created().json(ApiResponse::success(point, "Checkout point created")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create checkout point")),
    }
}

pub async fn update_checkout_point(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    point_req: web::Json<UpdatePointRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    let id = path.into_inner();
    
    match sqlx::query_as::<_, CheckoutPoint>(
        r#"
        UPDATE checkout_points 
        SET latitude = $1, longitude = $2, radius = $3, location_name = $4, allowed_department = $5
        WHERE id = $6
        RETURNING *
        "#
    )
    .bind(point_req.latitude)
    .bind(point_req.longitude)
    .bind(point_req.radius)
    .bind(&point_req.location_name)
    .bind(&point_req.allowed_department)
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(point)) => HttpResponse::Ok().json(ApiResponse::success(point, "Checkout point updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("Checkout point not found")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update checkout point")),
    }
}

pub async fn delete_checkout_point(
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
    
    match sqlx::query("DELETE FROM checkout_points WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Checkout point deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Checkout point not found"))
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete checkout point")),
    }
}