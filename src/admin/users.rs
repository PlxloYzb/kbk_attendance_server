use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{CreateUserRequest, UpdateUserRequest};
use crate::models::{ApiResponse, UserInfo};

pub async fn get_users(
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

    match sqlx::query_as::<_, UserInfo>("SELECT * FROM user_info ORDER BY id")
        .fetch_all(pool.as_ref())
        .await
    {
        Ok(users) => HttpResponse::Ok().json(ApiResponse::success(users, "Users retrieved")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve users")),
    }
}

pub async fn create_user(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    user_req: web::Json<CreateUserRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    match sqlx::query_as::<_, UserInfo>(
        r#"
        INSERT INTO user_info (user_id, department, department_name, department_code, passkey)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(&user_req.user_id)
    .bind(user_req.department)
    .bind(&user_req.department_name)
    .bind(&user_req.department_code)
    .bind(&user_req.passkey)
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(user) => HttpResponse::Created().json(ApiResponse::success(user, "User created")),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            HttpResponse::BadRequest().json(ApiResponse::<()>::error("User ID already exists"))
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create user")),
    }
}

pub async fn update_user(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    user_req: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Admin access required"));
    }

    let id = path.into_inner();
    
    match sqlx::query_as::<_, UserInfo>(
        r#"
        UPDATE user_info 
        SET user_id = $1, department = $2, department_name = $3, department_code = $4, passkey = $5
        WHERE id = $6
        RETURNING *
        "#
    )
    .bind(&user_req.user_id)
    .bind(user_req.department)
    .bind(&user_req.department_name)
    .bind(&user_req.department_code)
    .bind(&user_req.passkey)
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse::success(user, "User updated")),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found")),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            HttpResponse::BadRequest().json(ApiResponse::<()>::error("User ID already exists"))
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update user")),
    }
}

pub async fn delete_user(
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
    
    match sqlx::query("DELETE FROM user_info WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "User deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found"))
            }
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete user")),
    }
}