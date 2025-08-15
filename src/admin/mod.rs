pub mod auth;
pub mod models;
pub mod points;
pub mod users;
pub mod checkins;
pub mod stats;
pub mod admin_users;
pub mod time_settings;

use actix_web::web;

pub fn admin_routes() -> actix_web::Scope {
    web::scope("/admin")
        .route("/login", web::post().to(auth::admin_login))
        .route("/me", web::get().to(auth::get_admin_info))
        .service(
            web::scope("")
                .service(
                    web::scope("/points")
                        .route("/checkin", web::get().to(points::get_checkin_points))
                        .route("/checkin", web::post().to(points::create_checkin_point))
                        .route("/checkin/{id}", web::put().to(points::update_checkin_point))
                        .route("/checkin/{id}", web::delete().to(points::delete_checkin_point))
                        .route("/checkout", web::get().to(points::get_checkout_points))
                        .route("/checkout", web::post().to(points::create_checkout_point))
                        .route("/checkout/{id}", web::put().to(points::update_checkout_point))
                        .route("/checkout/{id}", web::delete().to(points::delete_checkout_point))
                )
                .service(
                    web::scope("/users")
                        .route("", web::get().to(users::get_users))
                        .route("", web::post().to(users::create_user))
                        .route("/{id}", web::put().to(users::update_user))
                        .route("/{id}", web::delete().to(users::delete_user))
                )
                .service(
                    web::scope("/checkins")
                        .route("", web::get().to(checkins::get_checkins))
                        .route("", web::post().to(checkins::create_checkin))
                        .route("/{id}", web::put().to(checkins::update_checkin))
                        .route("/{id}", web::delete().to(checkins::delete_checkin))
                )
                .service(
                    web::scope("/stats")
                        .route("/department", web::get().to(stats::get_department_stats))
                        .route("/department/filtered", web::get().to(stats::get_filtered_department_stats))
                        .route("/user-detail", web::get().to(stats::get_user_detail))
                        .route("/export", web::get().to(stats::export_attendance_csv))
                )
                .service(
                    web::scope("/admin-users")
                        .route("", web::get().to(admin_users::get_admin_users))
                        .route("", web::post().to(admin_users::create_admin_user))
                        .route("/{id}", web::put().to(admin_users::update_admin_user))
                        .route("/{id}", web::delete().to(admin_users::delete_admin_user))
                        .route("/{id}/password", web::put().to(admin_users::reset_admin_password))
                )
                .service(
                    web::scope("/time-settings")
                        .route("/users", web::get().to(time_settings::get_users_with_time_settings))
                        .route("/batch", web::post().to(time_settings::batch_update_time_settings))
                        .route("/{user_id}", web::delete().to(time_settings::delete_time_setting))
                )
        )
}