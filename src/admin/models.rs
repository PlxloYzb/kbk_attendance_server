use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub department: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLoginResponse {
    pub token: String,
    pub user: AdminUserInfo,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AdminUserInfo {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub department: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePointRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub location_name: String,
    pub allowed_department: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePointRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub radius: f64,
    pub location_name: String,
    pub allowed_department: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub user_id: String,
    pub user_name: Option<String>,
    pub department: i32,
    pub department_name: Option<String>,
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub user_id: String,
    pub user_name: Option<String>,
    pub department: i32,
    pub department_name: Option<String>,
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCheckinRequest {
    pub user_id: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_synced: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheckinRequest {
    pub user_id: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_synced: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentStatsResponse {
    pub departments: Vec<DepartmentStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentStat {
    pub department: i32,
    pub department_name: Option<String>,
    pub user_count: i64,
    pub total_attendance_days: i64,
    pub avg_work_hours: f64,
    pub users: Vec<UserAttendanceStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAttendanceStat {
    pub user_id: String,
    pub user_name: Option<String>,
    pub total_days: i64,
    pub total_hours: f64,
    pub last_checkin: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserResponse {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub department: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAdminUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub department: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAdminUserRequest {
    pub username: String,
    pub password: Option<String>,
    pub role: String,
    pub department: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilteredDepartmentStatsRequest {
    pub month: Option<u32>,
    pub year: Option<i32>,
    pub user_name: Option<String>,
    pub department: Option<i32>,
    pub view_type: Option<String>, // "month" or "year"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailRequest {
    pub user_id: String,
    pub month: u32,
    pub year: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailResponse {
    pub user_id: String,
    pub user_name: Option<String>,
    pub month: u32,
    pub year: i32,
    pub total_days: i64,
    pub total_hours: f64,
    pub records: Vec<UserDetailRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetailRecord {
    pub date: chrono::NaiveDate,
    pub first_checkin: Option<DateTime<Utc>>,
    pub last_checkout: Option<DateTime<Utc>>,
    pub total_work_minutes: Option<i32>,
    pub total_sessions: Option<i32>,
    pub is_late: bool,
    pub is_early_leave: bool,
}

#[derive(Debug, Clone)]
pub struct AdminSession {
    pub user_id: i32,
    pub username: String,
    pub role: String,
    pub department: Option<i32>,
}