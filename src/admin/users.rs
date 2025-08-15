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

    let mut transaction = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error"));
        }
    };

    // Create user
    let user_result = sqlx::query_as::<_, UserInfo>(
        r#"
        INSERT INTO user_info (user_id, user_name, department, department_name, passkey)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(&user_req.user_id)
    .bind(&user_req.user_name)
    .bind(user_req.department)
    .bind(&user_req.department_name)
    .bind(&user_req.passkey)
    .fetch_one(&mut *transaction)
    .await;

    match user_result {
        Ok(user) => {
            // Create default time settings for the new user
            let time_settings_result = sqlx::query(
                "INSERT INTO user_time_settings (user_id, on_duty_time, off_duty_time) VALUES ($1, '07:30:00', '17:00:00')"
            )
            .bind(&user_req.user_id)
            .execute(&mut *transaction)
            .await;

            match time_settings_result {
                Ok(_) => {
                    if let Err(e) = transaction.commit().await {
                        log::error!("Failed to commit transaction: {:?}", e);
                        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to save user"));
                    }
                    HttpResponse::Created().json(ApiResponse::success(user, "User created successfully"))
                }
                Err(e) => {
                    log::error!("Failed to create time settings for user: {:?}", e);
                    if let Err(rollback_err) = transaction.rollback().await {
                        log::error!("Failed to rollback transaction: {:?}", rollback_err);
                    }
                    HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create user time settings"))
                }
            }
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            if let Err(rollback_err) = transaction.rollback().await {
                log::error!("Failed to rollback transaction: {:?}", rollback_err);
            }
            HttpResponse::BadRequest().json(ApiResponse::<()>::error("User ID already exists"))
        }
        Err(e) => {
            log::error!("Failed to create user: {:?}", e);
            if let Err(rollback_err) = transaction.rollback().await {
                log::error!("Failed to rollback transaction: {:?}", rollback_err);
            }
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create user"))
        }
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
        SET user_id = $1, user_name = $2, department = $3, department_name = $4, passkey = $5
        WHERE id = $6
        RETURNING *
        "#
    )
    .bind(&user_req.user_id)
    .bind(&user_req.user_name)
    .bind(user_req.department)
    .bind(&user_req.department_name)
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