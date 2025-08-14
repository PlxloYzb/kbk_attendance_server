use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{AdminUser, AdminUserResponse, CreateAdminUserRequest, UpdateAdminUserRequest, ResetPasswordRequest};
use crate::models::ApiResponse;

pub async fn get_admin_users(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Only admin role can manage admin users
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"));
    }

    match sqlx::query_as::<_, AdminUser>(
        "SELECT * FROM admin_user ORDER BY username"
    )
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(admin_users) => {
            // Convert to response format (without passwords)
            let response: Vec<AdminUserResponse> = admin_users.into_iter().map(|user| AdminUserResponse {
                id: user.id,
                username: user.username,
                role: user.role,
                department: user.department,
                created_at: user.created_at,
            }).collect();
            
            HttpResponse::Ok().json(ApiResponse::success(response, "Admin users retrieved"))
        }
        Err(e) => {
            log::error!("Failed to retrieve admin users: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve admin users"))
        }
    }
}

pub async fn create_admin_user(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    user_req: web::Json<CreateAdminUserRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Only admin role can create admin users
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"));
    }

    // Validate department requirement for department role
    if user_req.role == "department" && user_req.department.is_none() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error("Department is required for department role"));
    }

    match sqlx::query_as::<_, AdminUser>(
        r#"
        INSERT INTO admin_user (username, password, role, department)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(&user_req.username)
    .bind(&user_req.password)
    .bind(&user_req.role)
    .bind(&user_req.department)
    .fetch_one(pool.as_ref())
    .await
    {
        Ok(admin_user) => {
            let response = AdminUserResponse {
                id: admin_user.id,
                username: admin_user.username,
                role: admin_user.role,
                department: admin_user.department,
                created_at: admin_user.created_at,
            };
            HttpResponse::Created().json(ApiResponse::success(response, "Admin user created"))
        }
        Err(e) => {
            log::error!("Failed to create admin user: {:?}", e);
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                HttpResponse::Conflict().json(ApiResponse::<()>::error("Username already exists"))
            } else {
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create admin user"))
            }
        }
    }
}

pub async fn update_admin_user(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    user_req: web::Json<UpdateAdminUserRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    let id = path.into_inner();

    // Only admin role can update admin users
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"));
    }

    // Validate department requirement for department role
    if user_req.role == "department" && user_req.department.is_none() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error("Department is required for department role"));
    }

    // Build update query dynamically based on whether password is provided
    let query = if user_req.password.is_some() {
        r#"
        UPDATE admin_user 
        SET username = $1, password = $2, role = $3, department = $4
        WHERE id = $5
        RETURNING *
        "#
    } else {
        r#"
        UPDATE admin_user 
        SET username = $1, role = $2, department = $3
        WHERE id = $4
        RETURNING *
        "#
    };

    let result = if let Some(ref password) = user_req.password {
        sqlx::query_as::<_, AdminUser>(query)
            .bind(&user_req.username)
            .bind(password)
            .bind(&user_req.role)
            .bind(&user_req.department)
            .bind(id)
            .fetch_optional(pool.as_ref())
            .await
    } else {
        sqlx::query_as::<_, AdminUser>(query)
            .bind(&user_req.username)
            .bind(&user_req.role)
            .bind(&user_req.department)
            .bind(id)
            .fetch_optional(pool.as_ref())
            .await
    };

    match result {
        Ok(Some(admin_user)) => {
            let response = AdminUserResponse {
                id: admin_user.id,
                username: admin_user.username,
                role: admin_user.role,
                department: admin_user.department,
                created_at: admin_user.created_at,
            };
            HttpResponse::Ok().json(ApiResponse::success(response, "Admin user updated"))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Admin user not found"))
        }
        Err(e) => {
            log::error!("Failed to update admin user: {:?}", e);
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                HttpResponse::Conflict().json(ApiResponse::<()>::error("Username already exists"))
            } else {
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to update admin user"))
            }
        }
    }
}

pub async fn delete_admin_user(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    let id = path.into_inner();

    // Only admin role can delete admin users
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"));
    }

    // Prevent deleting your own account
    if session.user_id == id {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error("Cannot delete your own account"));
    }

    match sqlx::query("DELETE FROM admin_user WHERE id = $1")
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Admin user deleted"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Admin user not found"))
            }
        }
        Err(e) => {
            log::error!("Failed to delete admin user: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to delete admin user"))
        }
    }
}

pub async fn reset_admin_password(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    password_req: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    let id = path.into_inner();

    // Only admin role can reset passwords
    if session.role != "admin" {
        return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Insufficient permissions"));
    }

    match sqlx::query("UPDATE admin_user SET password = $1 WHERE id = $2")
        .bind(&password_req.new_password)
        .bind(id)
        .execute(pool.as_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::<()>::success((), "Password reset successfully"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Admin user not found"))
            }
        }
        Err(e) => {
            log::error!("Failed to reset admin password: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to reset password"))
        }
    }
}