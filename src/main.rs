mod admin;
mod auth;
mod db;
mod handlers;
mod models;
mod sync;
mod timezone_config;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer, HttpResponse};
use actix_files as fs;
use dotenv::dotenv;
use env_logger::Env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    log::info!("Starting KBK Attendance Server...");

    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    log::info!("Database connection established");

    // Initialize sync service
    let sync_service = Arc::new(sync::SyncService::new(Arc::new(pool.clone())));
    
    // Run startup sync to ensure all users have time settings
    sync_service.startup_sync().await;
    
    // Start periodic sync process
    sync_service.clone().start_periodic_sync();

    let server_port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let server_addr = format!("0.0.0.0:{}", server_port);

    log::info!("Server starting on {}", server_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/", web::get().to(|| async {
                HttpResponse::Found()
                    .append_header(("Location", "/ui/login.html"))
                    .finish()
            }))
            .service(
                web::scope("/api")
                    .route("/auth/verify", web::post().to(handlers::verify_auth))
                    .route("/points/checkin", web::post().to(handlers::get_checkin_points))
                    .route("/points/checkout", web::post().to(handlers::get_checkout_points))
                    .route("/checkin/sync", web::post().to(handlers::sync_checkins))
                    .route("/checkin/count", web::post().to(handlers::check_count))
                    .route("/checkin/full-sync", web::post().to(handlers::full_sync))
                    .route("/stats/monthly", web::post().to(handlers::get_monthly_stats))
                    .route("/sessions/daily", web::post().to(handlers::get_daily_sessions))
            )
            .service(admin::admin_routes())
            .service(fs::Files::new("/ui", "./src/ui").index_file("login.html"))
    })
    .bind(&server_addr)?
    .run()
    .await
}