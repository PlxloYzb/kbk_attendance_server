use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{DepartmentStatsResponse, DepartmentStat, UserAttendanceStat};
use crate::models::ApiResponse;

pub async fn get_department_stats(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    let departments_query = if session.role == "admin" {
        // Admin can see all departments
        r#"
        SELECT 
            ui.department,
            ui.department_name,
            COUNT(DISTINCT ui.user_id) as user_count,
            COUNT(DISTINCT ats.date) as total_attendance_days,
            CAST(COALESCE(AVG(ats.total_work_minutes), 0) / 60.0 AS FLOAT8) as avg_work_hours
        FROM user_info ui
        LEFT JOIN attendance_summary ats ON ui.user_id = ats.user_id
        GROUP BY ui.department, ui.department_name
        ORDER BY ui.department
        "#
    } else {
        // Department users can only see their own department
        r#"
        SELECT 
            ui.department,
            ui.department_name,
            COUNT(DISTINCT ui.user_id) as user_count,
            COUNT(DISTINCT ats.date) as total_attendance_days,
            CAST(COALESCE(AVG(ats.total_work_minutes), 0) / 60.0 AS FLOAT8) as avg_work_hours
        FROM user_info ui
        LEFT JOIN attendance_summary ats ON ui.user_id = ats.user_id
        WHERE ui.department = $1
        GROUP BY ui.department, ui.department_name
        ORDER BY ui.department
        "#
    };

    let departments_result = if session.role == "admin" {
        sqlx::query_as::<_, (i32, Option<String>, i64, i64, f64)>(departments_query)
            .fetch_all(pool.as_ref())
            .await
    } else {
        sqlx::query_as::<_, (i32, Option<String>, i64, i64, f64)>(departments_query)
            .bind(session.department.unwrap_or(0))
            .fetch_all(pool.as_ref())
            .await
    };

    let departments = match departments_result {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Failed to retrieve department stats: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve department stats"));
        }
    };

    let mut dept_stats = Vec::new();

    for (department, department_name, user_count, total_attendance_days, avg_work_hours) in departments {
        // Get individual user stats for this department
        let users_query = r#"
            SELECT 
                ui.user_id,
                ui.user_name,
                COUNT(DISTINCT ats.date) as total_days,
                CAST(COALESCE(SUM(ats.total_work_minutes), 0) / 60.0 AS FLOAT8) as total_hours,
                MAX(ats.last_checkout_time) as last_checkin
            FROM user_info ui
            LEFT JOIN attendance_summary ats ON ui.user_id = ats.user_id
            WHERE ui.department = $1
            GROUP BY ui.user_id, ui.user_name
            ORDER BY ui.user_id
        "#;

        let users_result = sqlx::query_as::<_, (String, Option<String>, i64, f64, Option<chrono::DateTime<chrono::Utc>>)>(users_query)
            .bind(department)
            .fetch_all(pool.as_ref())
            .await;

        let users = match users_result {
            Ok(rows) => rows.into_iter().map(|(user_id, user_name, total_days, total_hours, last_checkin)| {
                UserAttendanceStat {
                    user_id,
                    user_name,
                    total_days,
                    total_hours,
                    last_checkin,
                }
            }).collect(),
            Err(e) => {
                log::error!("Failed to retrieve user stats for department {}: {:?}", department, e);
                Vec::new()
            },
        };

        dept_stats.push(DepartmentStat {
            department,
            department_name,
            user_count,
            total_attendance_days,
            avg_work_hours,
            users,
        });
    }

    let response = DepartmentStatsResponse {
        departments: dept_stats,
    };

    HttpResponse::Ok().json(ApiResponse::success(response, "Department statistics retrieved"))
}

pub async fn export_attendance_csv(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    _query: web::Query<ExportQuery>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    let export_query = if session.role == "admin" {
        // Admin can export all departments
        r#"
        SELECT 
            ats.user_id,
            ats.date,
            ats.first_checkin_time,
            ats.last_checkout_time,
            ats.total_work_minutes,
            ats.total_sessions,
            ui.department,
            ui.department_name
        FROM attendance_summary ats
        JOIN user_info ui ON ats.user_id = ui.user_id
        ORDER BY ats.date DESC, ats.user_id
        "#
    } else {
        // Department users can only export their department
        r#"
        SELECT 
            ats.user_id,
            ats.date,
            ats.first_checkin_time,
            ats.last_checkout_time,
            ats.total_work_minutes,
            ats.total_sessions,
            ui.department,
            ui.department_name
        FROM attendance_summary ats
        JOIN user_info ui ON ats.user_id = ui.user_id
        WHERE ui.department = $1
        ORDER BY ats.date DESC, ats.user_id
        "#
    };

    let records_result = if session.role == "admin" {
        sqlx::query_as::<_, (String, chrono::NaiveDate, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, i32, i32, i32, Option<String>)>(export_query)
            .fetch_all(pool.as_ref())
            .await
    } else {
        sqlx::query_as::<_, (String, chrono::NaiveDate, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, i32, i32, i32, Option<String>)>(export_query)
            .bind(session.department.unwrap_or(0))
            .fetch_all(pool.as_ref())
            .await
    };

    let records = match records_result {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Failed to retrieve attendance data for export: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve attendance data"));
        }
    };

    // Generate CSV content
    let mut csv_content = String::from("User ID,Date,First Checkin,Last Checkout,Work Minutes,Work Hours,Sessions,Department,Department Name\n");
    
    for (user_id, date, first_checkin, last_checkout, work_minutes, sessions, department, department_name) in records {
        let first_checkin_str = first_checkin
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "".to_string());
        
        let last_checkout_str = last_checkout
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "".to_string());
        
        let work_hours = work_minutes as f64 / 60.0;
        let dept_name = department_name.unwrap_or_else(|| "".to_string());
        
        csv_content.push_str(&format!(
            "{},{},{},{},{},{:.2},{},{},{}\n",
            user_id, date, first_checkin_str, last_checkout_str, work_minutes, work_hours, sessions, department, dept_name
        ));
    }

    let filename = if let Some(dept) = session.department {
        if session.role == "department" {
            format!("attendance_department_{}.csv", dept)
        } else {
            "attendance_export.csv".to_string()
        }
    } else {
        "attendance_export.csv".to_string()
    };

    HttpResponse::Ok()
        .content_type("text/csv")
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(csv_content)
}

#[derive(serde::Deserialize)]
pub struct ExportQuery {
    pub _department: Option<i32>,
    pub _start_date: Option<chrono::NaiveDate>,
    pub _end_date: Option<chrono::NaiveDate>,
}