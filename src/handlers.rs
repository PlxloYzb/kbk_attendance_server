use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::auth::{verify_passkey, verify_user_passkey};
use crate::models::*;

pub async fn verify_auth(
    pool: web::Data<PgPool>,
    req: web::Json<AuthRequest>,
) -> HttpResponse {
    match verify_passkey(&pool, &req.passkey).await {
        Ok(Some(user)) => {
            let response = UserInfoResponse {
                user_id: user.user_id,
                user_name: user.user_name,
                department: user.department,
                department_name: user.department_name,
            };
            HttpResponse::Ok().json(ApiResponse::success(response, "Authentication successful"))
        }
        Ok(None) => {
            HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid passkey"))
        }
        Err(_) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error"))
        }
    }
}

pub async fn get_checkin_points(
    pool: web::Data<PgPool>,
    req: web::Json<AuthRequest>,
) -> HttpResponse {
    let user = match verify_passkey(&pool, &req.passkey).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid passkey")),
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error")),
    };

    match sqlx::query_as::<_, CheckinPoint>(
        "SELECT * FROM checkin_points WHERE $1 = ANY(allowed_department) OR 0 = ANY(allowed_department)"
    )
    .bind(user.department)
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(points) => HttpResponse::Ok().json(ApiResponse::success(points, "Checkin points retrieved")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve checkin points")),
    }
}

pub async fn get_checkout_points(
    pool: web::Data<PgPool>,
    req: web::Json<AuthRequest>,
) -> HttpResponse {
    let user = match verify_passkey(&pool, &req.passkey).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid passkey")),
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Database error")),
    };

    match sqlx::query_as::<_, CheckoutPoint>(
        "SELECT * FROM checkout_points WHERE $1 = ANY(allowed_department) OR 0 = ANY(allowed_department)"
    )
    .bind(user.department)
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(points) => HttpResponse::Ok().json(ApiResponse::success(points, "Checkout points retrieved")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve checkout points")),
    }
}

pub async fn sync_checkins(
    pool: web::Data<PgPool>,
    req: web::Json<SyncRequest>,
) -> HttpResponse {
    if !verify_user_passkey(&pool, &req.user_id, &req.passkey).await.unwrap_or(false) {
        return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"));
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Transaction failed")),
    };

    // First, insert all checkins to the checkins table
    for checkin in &req.checkins {
        let result = sqlx::query(
            "INSERT INTO checkins (user_id, action, created_at, latitude, longitude, is_synced) 
             VALUES ($1, $2, $3, $4, $5, 1)"
        )
        .bind(&req.user_id)
        .bind(&checkin.action)
        .bind(&checkin.created_at)
        .bind(checkin.latitude)
        .bind(checkin.longitude)
        .execute(&mut *tx)
        .await;

        if result.is_err() {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to sync checkins"));
        }
    }

    // Group checkins by date and process sessions
    use std::collections::HashMap;
    let mut checkins_by_date: HashMap<NaiveDate, Vec<&CheckinData>> = HashMap::new();
    
    for checkin in &req.checkins {
        let date = checkin.created_at.date_naive();
        checkins_by_date.entry(date).or_insert_with(Vec::new).push(checkin);
    }

    // Process each day's checkins to create sessions
    for (date, day_checkins) in checkins_by_date {
        // Sort checkins by time
        let mut sorted_checkins = day_checkins.clone();
        sorted_checkins.sort_by_key(|c| c.created_at);

        // Track open session
        let mut open_session: Option<&CheckinData> = None;
        let mut session_number = 
            match sqlx::query_scalar::<_, i32>(
                "SELECT COALESCE(MAX(session_number), 0) FROM attendance_sessions 
                 WHERE user_id = $1 AND date = $2"
            )
            .bind(&req.user_id)
            .bind(date)
            .fetch_one(&mut *tx)
            .await {
                Ok(num) => num,
                Err(_) => 0,
            };

        // Check for incomplete session from previous day (midnight crossing fix)
        let _has_previous_incomplete = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM attendance_sessions 
             WHERE user_id = $1 AND date = $2 - INTERVAL '1 day' AND checkout_time IS NULL
               AND checkin_time > $2 - INTERVAL '1 day' + INTERVAL '16 hours')"
        )
        .bind(&req.user_id)
        .bind(date)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(false);

        for checkin in sorted_checkins {
            if checkin.action == "IN" {
                if open_session.is_none() {
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
                    .bind(&req.user_id)
                    .bind(date)
                    .bind(session_number)
                    .bind(checkin.created_at)
                    .bind(checkin.latitude)
                    .bind(checkin.longitude)
                    .execute(&mut *tx)
                    .await;

                    if result.is_err() {
                        let _ = tx.rollback().await;
                        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create session"));
                    }
                    open_session = Some(checkin);
                }
                // If there's already an open session, ignore this IN (or log warning)
            } else if checkin.action == "OUT" {
                if open_session.is_some() {
                    // Close the current session
                    let result = sqlx::query(
                        "UPDATE attendance_sessions 
                         SET checkout_time = $1, checkout_latitude = $2, checkout_longitude = $3
                         WHERE user_id = $4 AND date = $5 AND session_number = $6"
                    )
                    .bind(checkin.created_at)
                    .bind(checkin.latitude)
                    .bind(checkin.longitude)
                    .bind(&req.user_id)
                    .bind(date)
                    .bind(session_number)
                    .execute(&mut *tx)
                    .await;

                    if result.is_err() {
                        let _ = tx.rollback().await;
                        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to close session"));
                    }
                    open_session = None;
                } else {
                    // OUT without IN - check for incomplete session from previous day first (midnight crossing)
                    let prev_day_updated = sqlx::query(
                        "UPDATE attendance_sessions 
                         SET checkout_time = $1, checkout_latitude = $2, checkout_longitude = $3
                         WHERE user_id = $4 AND date = $5 - INTERVAL '1 day' AND checkout_time IS NULL
                           AND checkin_time < $1 AND checkin_time > $1 - INTERVAL '16 hours'
                         ORDER BY checkin_time DESC LIMIT 1"
                    )
                    .bind(checkin.created_at)
                    .bind(checkin.latitude)
                    .bind(checkin.longitude)
                    .bind(&req.user_id)
                    .bind(date)
                    .execute(&mut *tx)
                    .await;

                    // If no previous day session was updated, create checkout-only session
                    if prev_day_updated.map(|r| r.rows_affected()).unwrap_or(0) == 0 {
                        session_number += 1;
                        let result = sqlx::query(
                            "INSERT INTO attendance_sessions 
                             (user_id, date, session_number, checkin_time, checkout_time, checkout_latitude, checkout_longitude) 
                             VALUES ($1, $2, $3, $4, $5, $6, $7)
                             ON CONFLICT (user_id, date, session_number) DO NOTHING"
                        )
                        .bind(&req.user_id)
                        .bind(date)
                        .bind(session_number)
                        .bind(checkin.created_at) // Use checkout time as checkin for incomplete session
                        .bind(checkin.created_at)
                        .bind(checkin.latitude)
                        .bind(checkin.longitude)
                        .execute(&mut *tx)
                        .await;

                        if result.is_err() {
                            let _ = tx.rollback().await;
                            return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to create incomplete session"));
                        }
                    }
                }
            }
        }
    }

    match tx.commit().await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success(
            req.checkins.len(),
            "Checkins synced successfully"
        )),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to commit transaction")),
    }
}

pub async fn check_count(
    pool: web::Data<PgPool>,
    req: web::Json<CountRequest>,
) -> HttpResponse {
    if !verify_user_passkey(&pool, &req.user_id, &req.passkey).await.unwrap_or(false) {
        return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"));
    }

    let result = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM checkins WHERE user_id = $1"
    )
    .bind(&req.user_id)
    .fetch_one(pool.as_ref())
    .await;

    match result {
        Ok(record) => {
            let server_count = record.0;
            let action = if server_count == req.local_count {
                "none"
            } else if server_count < req.local_count {
                "sync"
            } else {
                "full_sync"
            };

            let response = CountResponse {
                action: action.to_string(),
                server_count,
            };
            HttpResponse::Ok().json(ApiResponse::success(response, "Count check completed"))
        }
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to check count")),
    }
}

pub async fn full_sync(
    pool: web::Data<PgPool>,
    req: web::Json<FullSyncRequest>,
) -> HttpResponse {
    if !verify_user_passkey(&pool, &req.user_id, &req.passkey).await.unwrap_or(false) {
        return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"));
    }

    match sqlx::query_as::<_, Checkin>(
        "SELECT * FROM checkins WHERE user_id = $1 ORDER BY created_at ASC"
    )
    .bind(&req.user_id)
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(checkins) => HttpResponse::Ok().json(ApiResponse::success(checkins, "Full sync completed")),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve checkins")),
    }
}

pub async fn get_monthly_stats(
    pool: web::Data<PgPool>,
    req: web::Json<MonthlyStatsRequest>,
) -> HttpResponse {
    if !verify_user_passkey(&pool, &req.user_id, &req.passkey).await.unwrap_or(false) {
        return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"));
    }

    let start_date = NaiveDate::from_ymd_opt(req.year, req.month, 1).unwrap();
    let end_date = if req.month == 12 {
        NaiveDate::from_ymd_opt(req.year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(req.year, req.month + 1, 1).unwrap()
    };

    let rows = sqlx::query(
        r#"
        SELECT 
            ats.date,
            ats.first_checkin_time as checkin_time,
            ats.last_checkout_time as checkout_time,
            ats.total_work_minutes,
            ats.total_sessions,
            CASE 
                WHEN ats.first_checkin_time IS NOT NULL AND 
                     ats.first_checkin_time::time > COALESCE(uts.on_duty_time, '09:00:00'::time) THEN true
                ELSE false
            END as is_late,
            CASE 
                WHEN ats.last_checkout_time IS NOT NULL AND 
                     ats.last_checkout_time::time < COALESCE(uts.off_duty_time, '18:00:00'::time) THEN true
                ELSE false
            END as is_early_leave
        FROM attendance_summary ats
        LEFT JOIN user_time_settings uts ON ats.user_id = uts.user_id
        WHERE ats.user_id = $1 AND ats.date >= $2 AND ats.date < $3
        ORDER BY ats.date ASC
        "#
    )
    .bind(&req.user_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool.as_ref())
    .await;

    let attendance_records = match rows {
        Ok(rows) => {
            let mut records = Vec::new();
            for row in rows {
                use sqlx::Row;
                records.push(DailyAttendance {
                    date: row.get("date"),
                    checkin_time: row.get("checkin_time"),
                    checkout_time: row.get("checkout_time"),
                    is_late: row.get("is_late"),
                    is_early_leave: row.get("is_early_leave"),
                    total_work_minutes: row.get("total_work_minutes"),
                    total_sessions: row.get("total_sessions"),
                });
            }
            Ok(records)
        }
        Err(e) => Err(e),
    };

    match attendance_records {
        Ok(records) => {
            let attendance_days = records.iter().filter(|r| r.checkin_time.is_some()).count() as i32;
            let late_count = records.iter().filter(|r| r.is_late).count() as i32;
            let early_leave_count = records.iter().filter(|r| r.is_early_leave).count() as i32;

            let response = MonthlyStatsResponse {
                attendance_days,
                late_count,
                early_leave_count,
                details: records,
            };

            HttpResponse::Ok().json(ApiResponse::success(response, "Monthly stats retrieved"))
        }
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve stats")),
    }
}

pub async fn get_daily_sessions(
    pool: web::Data<PgPool>,
    req: web::Json<DailySessionsRequest>,
) -> HttpResponse {
    if !verify_user_passkey(&pool, &req.user_id, &req.passkey).await.unwrap_or(false) {
        return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("Invalid credentials"));
    }

    // Fetch all sessions for the given date
    let sessions = match sqlx::query_as::<_, AttendanceSession>(
        "SELECT * FROM attendance_sessions 
         WHERE user_id = $1 AND date = $2 
         ORDER BY session_number ASC"
    )
    .bind(&req.user_id)
    .bind(req.date)
    .fetch_all(pool.as_ref())
    .await {
        Ok(sessions) => sessions,
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve sessions")),
    };

    // Fetch the summary for the given date
    let summary = match sqlx::query_as::<_, AttendanceSummary>(
        "SELECT * FROM attendance_summary 
         WHERE user_id = $1 AND date = $2"
    )
    .bind(&req.user_id)
    .bind(req.date)
    .fetch_optional(pool.as_ref())
    .await {
        Ok(Some(summary)) => summary,
        Ok(None) => {
            // Create empty summary if none exists
            AttendanceSummary {
                id: 0,
                user_id: req.user_id.clone(),
                date: req.date,
                checkin_time: None,
                checkout_time: None,
                first_checkin_time: None,
                last_checkout_time: None,
                total_work_minutes: 0,
                total_break_minutes: None,
                total_sessions: 0,
                is_complete: false,
                updated_at: None,
            }
        },
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse::<()>::error("Failed to retrieve summary")),
    };

    let response = DailySessionsResponse {
        date: req.date,
        sessions,
        summary,
    };

    HttpResponse::Ok().json(ApiResponse::success(response, "Daily sessions retrieved"))
}