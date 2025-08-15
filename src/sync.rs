use sqlx::PgPool;
use tokio::time::{interval, Duration};
use std::sync::Arc;

/// Background sync service to ensure data consistency
pub struct SyncService {
    pool: Arc<PgPool>,
}

impl SyncService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Sync user_time_settings table with user_info table
    /// Ensures every user in user_info has a corresponding entry in user_time_settings
    pub async fn sync_user_time_settings(&self) -> Result<usize, sqlx::Error> {
        log::info!("Starting user time settings sync...");

        let result = sqlx::query(
            r#"
            INSERT INTO user_time_settings (user_id, on_duty_time, off_duty_time)
            SELECT 
                ui.user_id,
                '07:30:00'::time as on_duty_time,
                '17:00:00'::time as off_duty_time
            FROM user_info ui
            WHERE ui.user_id NOT IN (
                SELECT user_id FROM user_time_settings WHERE user_id IS NOT NULL
            )
            ON CONFLICT (user_id) DO NOTHING
            "#
        )
        .execute(self.pool.as_ref())
        .await?;

        let synced_count = result.rows_affected() as usize;
        
        if synced_count > 0 {
            log::info!("Synced {} users with default time settings", synced_count);
        } else {
            log::debug!("All users already have time settings - no sync needed");
        }

        // Verify sync completeness
        let verification: (i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM user_info) as total_users,
                (SELECT COUNT(*) FROM user_time_settings) as users_with_settings
            "#
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        if verification.0 != verification.1 {
            log::warn!(
                "Sync verification failed: {} users in user_info but {} users in user_time_settings",
                verification.0,
                verification.1
            );
        } else {
            log::info!("Sync verification passed: all users have time settings");
        }

        Ok(synced_count)
    }

    /// Run one-time sync during server startup
    pub async fn startup_sync(&self) {
        log::info!("Running startup sync for user time settings...");
        
        match self.sync_user_time_settings().await {
            Ok(count) => {
                if count > 0 {
                    log::info!("Startup sync completed: {} users synced", count);
                } else {
                    log::info!("Startup sync completed: all users already synced");
                }
            }
            Err(e) => {
                log::error!("Failed to sync user time settings during startup: {:?}", e);
            }
        }
    }

    /// Start periodic sync process (runs every hour)
    pub fn start_periodic_sync(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(3600)); // 1 hour
            
            loop {
                interval_timer.tick().await;
                
                log::debug!("Running periodic user time settings sync...");
                
                match self.sync_user_time_settings().await {
                    Ok(count) => {
                        if count > 0 {
                            log::info!("Periodic sync: {} users synced", count);
                        }
                    }
                    Err(e) => {
                        log::error!("Periodic sync failed: {:?}", e);
                    }
                }
            }
        });
    }

    /// Manual sync trigger (can be called from admin endpoints)
    pub async fn manual_sync(&self) -> Result<(usize, String), String> {
        match self.sync_user_time_settings().await {
            Ok(count) => {
                let message = if count > 0 {
                    format!("Successfully synced {} users with default time settings", count)
                } else {
                    "All users already have time settings - no sync needed".to_string()
                };
                Ok((count, message))
            }
            Err(e) => {
                let error_msg = format!("Failed to sync user time settings: {:?}", e);
                log::error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }
}
