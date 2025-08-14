use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Checkin {
    pub id: i32,
    pub user_id: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_synced: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckinRequest {
    pub user_id: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub user_id: String,
    pub passkey: String,
    pub checkins: Vec<CheckinData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckinData {
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CheckinPoint {
    pub id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub location_name: String,
    pub allowed_department: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CheckoutPoint {
    pub id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub location_name: String,
    pub allowed_department: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserInfo {
    pub id: i32,
    pub user_id: String,
    pub department: i32,
    pub department_name: Option<String>,
    pub department_code: Option<String>,
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub user_id: String,
    pub department: i32,
    pub department_name: Option<String>,
    pub department_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AttendanceSession {
    pub id: i32,
    pub user_id: String,
    pub date: NaiveDate,
    pub session_number: i32,
    pub checkin_time: DateTime<Utc>,
    pub checkout_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub checkin_latitude: Option<f64>,
    pub checkin_longitude: Option<f64>,
    pub checkout_latitude: Option<f64>,
    pub checkout_longitude: Option<f64>,
    pub checkin_location: Option<String>,
    pub checkout_location: Option<String>,
    pub is_complete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AttendanceSummary {
    pub id: i32,
    pub user_id: String,
    pub date: NaiveDate,
    pub checkin_time: Option<DateTime<Utc>>, // Backward compatibility
    pub checkout_time: Option<DateTime<Utc>>, // Backward compatibility
    pub first_checkin_time: Option<DateTime<Utc>>,
    pub last_checkout_time: Option<DateTime<Utc>>,
    pub total_work_minutes: i32,
    pub total_break_minutes: Option<i32>,
    pub total_sessions: i32,
    pub is_complete: bool,
    pub updated_at: Option<DateTime<Utc>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountRequest {
    pub user_id: String,
    pub passkey: String,
    pub local_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountResponse {
    pub action: String, // "none", "sync", "full_sync"
    pub server_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FullSyncRequest {
    pub user_id: String,
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyStatsRequest {
    pub user_id: String,
    pub passkey: String,
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyStatsResponse {
    pub attendance_days: i32,
    pub late_count: i32,
    pub early_leave_count: i32,
    pub details: Vec<DailyAttendance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyAttendance {
    pub date: NaiveDate,
    pub checkin_time: Option<DateTime<Utc>>,
    pub checkout_time: Option<DateTime<Utc>>,
    pub is_late: bool,
    pub is_early_leave: bool,
    pub total_work_minutes: Option<i32>,
    pub total_sessions: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySessionsRequest {
    pub user_id: String,
    pub passkey: String,
    pub date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySessionsResponse {
    pub date: NaiveDate,
    pub sessions: Vec<AttendanceSession>,
    pub summary: AttendanceSummary,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}