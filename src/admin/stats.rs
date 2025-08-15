use actix_web::{web, HttpResponse, HttpRequest};
use sqlx::PgPool;
use chrono::Datelike;

use crate::admin::auth::require_admin_auth;
use crate::admin::models::{
    DepartmentStatsResponse, DepartmentStat, UserAttendanceStat,
    FilteredDepartmentStatsRequest, UserDetailRequest, UserDetailResponse, UserDetailRecord
};
use crate::models::ApiResponse;
use crate::timezone_config::TimezoneConfig;

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
        // Use configured timezone for local display  
        let timezone_config = TimezoneConfig::local();
        
        let first_checkin_str = first_checkin
            .map(|dt| timezone_config.format_csv_datetime_with_tz(&dt))
            .unwrap_or_else(|| "".to_string());
        
        let last_checkout_str = last_checkout
            .map(|dt| timezone_config.format_csv_datetime_with_tz(&dt))
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

pub async fn get_filtered_department_stats(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<FilteredDepartmentStatsRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Get current date for default values
    let now = chrono::Utc::now();
    let current_year = now.year();
    let current_month = now.month();

    let target_year = query.year.unwrap_or(current_year);
    let target_month = query.month.unwrap_or(current_month);

    // Calculate date range for the selected month
    let start_date = chrono::NaiveDate::from_ymd_opt(target_year, target_month, 1)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap());
    
    let end_date = if target_month == 12 {
        chrono::NaiveDate::from_ymd_opt(target_year + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(target_year, target_month + 1, 1).unwrap()
    };

    // Build department filter conditions
    let mut dept_conditions = Vec::new();

    // Role-based department filtering
    if session.role == "department" {
        if let Some(dept) = session.department {
            dept_conditions.push(format!("ui.department = {}", dept));
        }
    } else if let Some(dept) = query.department {
        dept_conditions.push(format!("ui.department = {}", dept));
    }

    // User name filter
    if let Some(ref user_name) = query.user_name {
        if !user_name.trim().is_empty() {
            dept_conditions.push(format!("ui.user_name ILIKE '%{}%'", user_name.trim().replace("'", "''")));
        }
    }

    // Date range filtering
    dept_conditions.push(format!("ats.date >= '{}'", start_date));
    dept_conditions.push(format!("ats.date < '{}'", end_date));

    let where_clause = if dept_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", dept_conditions.join(" AND "))
    };

    let departments_query = format!(r#"
        SELECT 
            ui.department,
            ui.department_name,
            COUNT(DISTINCT ui.user_id) as user_count,
            COUNT(DISTINCT ats.date) as total_attendance_days,
            CAST(COALESCE(AVG(ats.total_work_minutes), 0) / 60.0 AS FLOAT8) as avg_work_hours
        FROM user_info ui
        LEFT JOIN attendance_summary ats ON ui.user_id = ats.user_id
        {}
        GROUP BY ui.department, ui.department_name
        ORDER BY ui.department
    "#, where_clause);

    let departments_result = sqlx::query_as::<_, (i32, Option<String>, i64, i64, f64)>(&departments_query)
        .fetch_all(pool.as_ref()).await;

    let departments = match departments_result {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Failed to retrieve filtered department stats: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve department stats"));
        }
    };

    let mut dept_stats = Vec::new();

    for (department, department_name, user_count, total_attendance_days, avg_work_hours) in departments {
        // Build user filter conditions for this department
        let mut user_conditions = vec![format!("ui.department = {}", department)];

        // User name filter
        if let Some(ref user_name) = query.user_name {
            if !user_name.trim().is_empty() {
                user_conditions.push(format!("ui.user_name ILIKE '%{}%'", user_name.trim().replace("'", "''")));
            }
        }

        let users_query = format!(r#"
            SELECT 
                ui.user_id,
                ui.user_name,
                COUNT(DISTINCT ats.date) as total_days,
                CAST(COALESCE(SUM(ats.total_work_minutes), 0) / 60.0 AS FLOAT8) as total_hours,
                MAX(ats.last_checkout_time) as last_checkin
            FROM user_info ui
            LEFT JOIN attendance_summary ats ON ui.user_id = ats.user_id 
                AND ats.date >= '{}' AND ats.date < '{}'
            WHERE {}
            GROUP BY ui.user_id, ui.user_name
            ORDER BY ui.user_id
        "#, start_date, end_date, user_conditions.join(" AND "));

        let users_result = sqlx::query_as::<_, (String, Option<String>, i64, f64, Option<chrono::DateTime<chrono::Utc>>)>(&users_query)
            .fetch_all(pool.as_ref()).await;

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

    HttpResponse::Ok().json(ApiResponse::success(response, "Filtered department statistics retrieved"))
}

pub async fn get_user_detail(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<UserDetailRequest>,
) -> HttpResponse {
    let session = match require_admin_auth(&req) {
        Ok(session) => session,
        Err(response) => return response,
    };

    // Check if user has permission to view this user's data
    let user_dept_query = "SELECT department, user_name FROM user_info WHERE user_id = $1";
    let user_info_result = sqlx::query_as::<_, (i32, Option<String>)>(user_dept_query)
        .bind(&query.user_id)
        .fetch_optional(pool.as_ref())
        .await;

    let (user_department, user_name) = match user_info_result {
        Ok(Some((dept, name))) => (dept, name),
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found"));
        }
        Err(e) => {
            log::error!("Failed to retrieve user info: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve user info"));
        }
    };

    // Permission check
    if session.role == "department" {
        if let Some(session_dept) = session.department {
            if session_dept != user_department {
                return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
            }
        } else {
            return HttpResponse::Forbidden().json(ApiResponse::<()>::error("Access denied"));
        }
    }

    // Calculate date range for the selected month
    let start_date = chrono::NaiveDate::from_ymd_opt(query.year, query.month, 1)
        .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(chrono::Utc::now().year(), chrono::Utc::now().month(), 1).unwrap());
    
    let end_date = if query.month == 12 {
        chrono::NaiveDate::from_ymd_opt(query.year + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(query.year, query.month + 1, 1).unwrap()
    };

    // Get detailed attendance records for the user and month
    let records_query = r#"
        SELECT 
            date,
            first_checkin_time,
            last_checkout_time,
            total_work_minutes,
            total_sessions,
            CASE 
                WHEN first_checkin_time IS NOT NULL AND EXTRACT(HOUR FROM first_checkin_time) > 9 THEN true
                ELSE false
            END as is_late,
            CASE 
                WHEN last_checkout_time IS NOT NULL AND EXTRACT(HOUR FROM last_checkout_time) < 18 THEN true
                ELSE false
            END as is_early_leave
        FROM attendance_summary
        WHERE user_id = $1 AND date >= $2 AND date < $3
        ORDER BY date ASC
    "#;

    let records_result = sqlx::query_as::<_, (chrono::NaiveDate, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, Option<i32>, Option<i32>, bool, bool)>(records_query)
        .bind(&query.user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool.as_ref())
        .await;

    let records = match records_result {
        Ok(rows) => rows.into_iter().map(|(date, first_checkin, last_checkout, total_work_minutes, total_sessions, is_late, is_early_leave)| {
            UserDetailRecord {
                date,
                first_checkin,
                last_checkout,
                total_work_minutes,
                total_sessions,
                is_late,
                is_early_leave,
            }
        }).collect::<Vec<_>>(),
        Err(e) => {
            log::error!("Failed to retrieve user detail records: {:?}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve user records"));
        }
    };

    // Calculate totals
    let total_days = records.iter().filter(|r| r.first_checkin.is_some()).count() as i64;
    let total_hours = records.iter()
        .map(|r| r.total_work_minutes.unwrap_or(0) as f64)
        .sum::<f64>() / 60.0;

    let response = UserDetailResponse {
        user_id: query.user_id.clone(),
        user_name,
        month: query.month,
        year: query.year,
        total_days,
        total_hours,
        records,
    };

    HttpResponse::Ok().json(ApiResponse::success(response, "User detail records retrieved"))
}