use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/kbk_attendance".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // Initialize database tables
    initialize_database(&pool).await?;
    
    Ok(pool)
}

async fn initialize_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Check if we need to migrate from old schema
    let needs_migration = check_needs_migration(pool).await?;
    
    if needs_migration {
        log::info!("Old schema detected, running migration...");
        migrate_to_session_schema(pool).await?;
    } else {
        log::info!("Initializing database with session-based schema...");
        create_session_schema(pool).await?;
    }
    
    // Create or update other tables
    create_base_tables(pool).await?;
    
    // Create indexes
    create_indexes(pool).await?;
    
    // Insert sample data if needed
    insert_sample_data(pool).await?;
    
    log::info!("Database initialization completed");
    Ok(())
}

async fn check_needs_migration(pool: &PgPool) -> Result<bool, sqlx::Error> {
    // Check if old attendance_info table exists without new columns
    let result = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_name = 'attendance_info'
        ) AND NOT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'attendance_info' 
            AND column_name = 'first_checkin_time'
        )
        "#
    )
    .fetch_one(pool)
    .await?;
    
    Ok(result)
}

async fn migrate_to_session_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create attendance_sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS attendance_sessions (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            date DATE NOT NULL,
            session_number INTEGER NOT NULL DEFAULT 1,
            checkin_time TIMESTAMP WITH TIME ZONE NOT NULL,
            checkout_time TIMESTAMP WITH TIME ZONE,
            duration_minutes INTEGER,
            checkin_latitude DOUBLE PRECISION,
            checkin_longitude DOUBLE PRECISION,
            checkout_latitude DOUBLE PRECISION,
            checkout_longitude DOUBLE PRECISION,
            checkin_location VARCHAR(255),
            checkout_location VARCHAR(255),
            is_complete BOOLEAN DEFAULT false,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            CONSTRAINT unique_user_date_session UNIQUE(user_id, date, session_number)
        )
        "#
    )
    .execute(pool)
    .await?;
    
    // Rename and enhance attendance_info to attendance_summary
    let _ = sqlx::query(
        r#"
        ALTER TABLE attendance_info RENAME TO attendance_summary
        "#
    )
    .execute(pool)
    .await;
    
    // Add new columns to attendance_summary
    sqlx::query(
        r#"
        ALTER TABLE attendance_summary 
            ADD COLUMN IF NOT EXISTS first_checkin_time TIMESTAMP WITH TIME ZONE,
            ADD COLUMN IF NOT EXISTS last_checkout_time TIMESTAMP WITH TIME ZONE,
            ADD COLUMN IF NOT EXISTS total_work_minutes INTEGER DEFAULT 0,
            ADD COLUMN IF NOT EXISTS total_break_minutes INTEGER DEFAULT 0,
            ADD COLUMN IF NOT EXISTS total_sessions INTEGER DEFAULT 0,
            ADD COLUMN IF NOT EXISTS is_complete BOOLEAN DEFAULT false,
            ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        "#
    )
    .execute(pool)
    .await?;
    
    // Create triggers and functions
    create_triggers_and_functions(pool).await?;
    
    // Migrate existing data from checkins table
    migrate_checkins_to_sessions(pool).await?;
    
    log::info!("Migration to session schema completed");
    Ok(())
}

async fn create_session_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create attendance_sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS attendance_sessions (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            date DATE NOT NULL,
            session_number INTEGER NOT NULL DEFAULT 1,
            checkin_time TIMESTAMP WITH TIME ZONE NOT NULL,
            checkout_time TIMESTAMP WITH TIME ZONE,
            duration_minutes INTEGER,
            checkin_latitude DOUBLE PRECISION,
            checkin_longitude DOUBLE PRECISION,
            checkout_latitude DOUBLE PRECISION,
            checkout_longitude DOUBLE PRECISION,
            checkin_location VARCHAR(255),
            checkout_location VARCHAR(255),
            is_complete BOOLEAN DEFAULT false,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            CONSTRAINT unique_user_date_session UNIQUE(user_id, date, session_number)
        )
        "#
    )
    .execute(pool)
    .await?;
    
    // Create attendance_summary table (new deployments use this name directly)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS attendance_summary (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            date DATE NOT NULL,
            checkin_time TIMESTAMP WITH TIME ZONE,
            checkout_time TIMESTAMP WITH TIME ZONE,
            first_checkin_time TIMESTAMP WITH TIME ZONE,
            last_checkout_time TIMESTAMP WITH TIME ZONE,
            total_work_minutes INTEGER DEFAULT 0,
            total_break_minutes INTEGER DEFAULT 0,
            total_sessions INTEGER DEFAULT 0,
            is_complete BOOLEAN DEFAULT false,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            UNIQUE(user_id, date)
        )
        "#
    )
    .execute(pool)
    .await?;
    
    // Create triggers and functions
    create_triggers_and_functions(pool).await?;
    
    Ok(())
}

async fn create_triggers_and_functions(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create function to update duration when checkout_time is set
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_session_duration()
        RETURNS TRIGGER AS $$
        BEGIN
            IF NEW.checkout_time IS NOT NULL AND NEW.checkin_time IS NOT NULL THEN
                NEW.duration_minutes := EXTRACT(EPOCH FROM (NEW.checkout_time - NEW.checkin_time)) / 60;
                NEW.is_complete := true;
            END IF;
            NEW.updated_at := NOW();
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql
        "#
    )
    .execute(pool)
    .await?;
    
    // Create trigger for automatic duration calculation
    sqlx::query(
        r#"
        DROP TRIGGER IF EXISTS update_session_duration_trigger ON attendance_sessions
        "#
    )
    .execute(pool)
    .await
    .ok();
    
    sqlx::query(
        r#"
        CREATE TRIGGER update_session_duration_trigger
            BEFORE UPDATE ON attendance_sessions
            FOR EACH ROW
            WHEN (OLD.checkout_time IS DISTINCT FROM NEW.checkout_time)
            EXECUTE FUNCTION update_session_duration()
        "#
    )
    .execute(pool)
    .await?;
    
    // Create function to update attendance_summary when sessions change
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_attendance_summary()
        RETURNS TRIGGER AS $$
        DECLARE
            v_first_checkin TIMESTAMP WITH TIME ZONE;
            v_last_checkout TIMESTAMP WITH TIME ZONE;
            v_total_minutes INTEGER;
            v_total_sessions INTEGER;
            v_is_complete BOOLEAN;
        BEGIN
            -- Calculate summary statistics for the day
            SELECT 
                MIN(checkin_time),
                MAX(checkout_time),
                COALESCE(SUM(duration_minutes), 0),
                COUNT(*),
                BOOL_AND(is_complete)
            INTO 
                v_first_checkin,
                v_last_checkout,
                v_total_minutes,
                v_total_sessions,
                v_is_complete
            FROM attendance_sessions
            WHERE user_id = COALESCE(NEW.user_id, OLD.user_id)
                AND date = COALESCE(NEW.date, OLD.date);

            -- Update or insert attendance_summary
            INSERT INTO attendance_summary (
                user_id, 
                date, 
                first_checkin_time, 
                last_checkout_time,
                checkin_time,
                checkout_time,
                total_work_minutes, 
                total_sessions, 
                is_complete,
                updated_at
            ) VALUES (
                COALESCE(NEW.user_id, OLD.user_id),
                COALESCE(NEW.date, OLD.date),
                v_first_checkin,
                v_last_checkout,
                v_first_checkin,
                v_last_checkout,
                v_total_minutes,
                v_total_sessions,
                v_is_complete,
                NOW()
            )
            ON CONFLICT (user_id, date) DO UPDATE SET
                first_checkin_time = EXCLUDED.first_checkin_time,
                last_checkout_time = EXCLUDED.last_checkout_time,
                checkin_time = EXCLUDED.checkin_time,
                checkout_time = EXCLUDED.checkout_time,
                total_work_minutes = EXCLUDED.total_work_minutes,
                total_sessions = EXCLUDED.total_sessions,
                is_complete = EXCLUDED.is_complete,
                updated_at = NOW();

            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql
        "#
    )
    .execute(pool)
    .await?;
    
    // Create trigger to update summary when sessions change
    sqlx::query(
        r#"
        DROP TRIGGER IF EXISTS update_summary_on_session_change ON attendance_sessions
        "#
    )
    .execute(pool)
    .await
    .ok();
    
    sqlx::query(
        r#"
        CREATE TRIGGER update_summary_on_session_change
            AFTER INSERT OR UPDATE OR DELETE ON attendance_sessions
            FOR EACH ROW
            EXECUTE FUNCTION update_attendance_summary()
        "#
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn migrate_checkins_to_sessions(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Check if we have data to migrate
    let checkin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM checkins")
        .fetch_one(pool)
        .await?;
    
    if checkin_count.0 > 0 {
        log::info!("Migrating {} checkin records to sessions...", checkin_count.0);
        
        sqlx::query(
            r#"
            INSERT INTO attendance_sessions (
                user_id,
                date,
                session_number,
                checkin_time,
                checkout_time,
                checkin_latitude,
                checkin_longitude,
                checkout_latitude,
                checkout_longitude,
                is_complete,
                duration_minutes
            )
            WITH checkin_pairs AS (
                SELECT 
                    ci.user_id,
                    ci.created_at::date as date,
                    ci.created_at as checkin_time,
                    ci.latitude as checkin_lat,
                    ci.longitude as checkin_lon,
                    (
                        SELECT co.created_at 
                        FROM checkins co 
                        WHERE co.user_id = ci.user_id 
                            AND co.action = 'OUT' 
                            AND co.created_at > ci.created_at
                            AND co.created_at::date = ci.created_at::date
                        ORDER BY co.created_at 
                        LIMIT 1
                    ) as checkout_time,
                    (
                        SELECT co.latitude 
                        FROM checkins co 
                        WHERE co.user_id = ci.user_id 
                            AND co.action = 'OUT' 
                            AND co.created_at > ci.created_at
                            AND co.created_at::date = ci.created_at::date
                        ORDER BY co.created_at 
                        LIMIT 1
                    ) as checkout_lat,
                    (
                        SELECT co.longitude 
                        FROM checkins co 
                        WHERE co.user_id = ci.user_id 
                            AND co.action = 'OUT' 
                            AND co.created_at > ci.created_at
                            AND co.created_at::date = ci.created_at::date
                        ORDER BY co.created_at 
                        LIMIT 1
                    ) as checkout_lon
                FROM checkins ci
                WHERE ci.action = 'IN'
            ),
            numbered_sessions AS (
                SELECT 
                    user_id,
                    date,
                    checkin_time,
                    checkout_time,
                    checkin_lat,
                    checkin_lon,
                    checkout_lat,
                    checkout_lon,
                    ROW_NUMBER() OVER (PARTITION BY user_id, date ORDER BY checkin_time) as session_num
                FROM checkin_pairs
            )
            SELECT 
                user_id,
                date,
                session_num as session_number,
                checkin_time,
                checkout_time,
                checkin_lat as checkin_latitude,
                checkin_lon as checkin_longitude,
                checkout_lat as checkout_latitude,
                checkout_lon as checkout_longitude,
                CASE WHEN checkout_time IS NOT NULL THEN true ELSE false END as is_complete,
                CASE 
                    WHEN checkout_time IS NOT NULL 
                    THEN EXTRACT(EPOCH FROM (checkout_time - checkin_time)) / 60
                    ELSE NULL 
                END as duration_minutes
            FROM numbered_sessions
            ON CONFLICT (user_id, date, session_number) DO NOTHING
            "#
        )
        .execute(pool)
        .await?;
        
        log::info!("Migration of checkin records completed");
    }
    
    Ok(())
}

async fn create_base_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create admin_user table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admin_user (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) UNIQUE NOT NULL,
            password VARCHAR(255) NOT NULL,
            role VARCHAR(50) NOT NULL CHECK (role IN ('admin', 'department')),
            department INTEGER,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create checkins table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS checkins (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            action VARCHAR(10) NOT NULL CHECK (action IN ('IN', 'OUT')),
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            latitude DOUBLE PRECISION,
            longitude DOUBLE PRECISION,
            is_synced INTEGER DEFAULT 0 CHECK (is_synced IN (0, 1))
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create checkin_points table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS checkin_points (
            id SERIAL PRIMARY KEY,
            latitude DOUBLE PRECISION NOT NULL,
            longitude DOUBLE PRECISION NOT NULL,
            radius DOUBLE PRECISION NOT NULL,
            location_name VARCHAR(255) NOT NULL,
            allowed_department INTEGER[] DEFAULT ARRAY[0]
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create checkout_points table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS checkout_points (
            id SERIAL PRIMARY KEY,
            latitude DOUBLE PRECISION NOT NULL,
            longitude DOUBLE PRECISION NOT NULL,
            radius DOUBLE PRECISION NOT NULL,
            location_name VARCHAR(255) NOT NULL,
            allowed_department INTEGER[] DEFAULT ARRAY[0]
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create user_info table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_info (
            id SERIAL PRIMARY KEY,
            user_id VARCHAR(255) UNIQUE NOT NULL,
            user_name VARCHAR(255),
            department INTEGER DEFAULT 99,
            department_name VARCHAR(100),
            passkey VARCHAR(255) NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn create_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Indexes for checkins table
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_checkins_user_id ON checkins(user_id)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_checkins_created_at ON checkins(created_at)")
        .execute(pool)
        .await?;
    
    // Indexes for attendance_sessions table
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_attendance_sessions_user_date ON attendance_sessions(user_id, date)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_attendance_sessions_date ON attendance_sessions(date)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_attendance_sessions_user_id ON attendance_sessions(user_id)")
        .execute(pool)
        .await?;
    
    // Indexes for attendance_summary table
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_attendance_summary_user_id ON attendance_summary(user_id)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_attendance_summary_date ON attendance_summary(date)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_attendance_summary_updated ON attendance_summary(updated_at)")
        .execute(pool)
        .await?;
    
    // Index for user_info table
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_info_passkey ON user_info(passkey)")
        .execute(pool)
        .await?;
    
    Ok(())
}

async fn insert_sample_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Check if we need to insert sample data
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_info")
        .fetch_one(pool)
        .await?;

    if user_count.0 == 0 {
        log::info!("No users found, inserting sample data...");
        

        // Insert sample checkin points
        sqlx::query(
            r#"
            INSERT INTO checkin_points (latitude, longitude, radius, location_name, allowed_department) 
            SELECT * FROM (VALUES
                (39.9042, 116.4074, 100, 'Office Building A', ARRAY[1, 99]),
                (39.9142, 116.4174, 150, 'Mining Site 1', ARRAY[2]),
                (39.9242, 116.4274, 200, 'Warehouse', ARRAY[5]),
                (39.9342, 116.4374, 100, 'Lab Building', ARRAY[6])
            ) AS v(latitude, longitude, radius, location_name, allowed_department)
            WHERE NOT EXISTS (SELECT 1 FROM checkin_points)
            "#
        )
        .execute(pool)
        .await?;

        // Insert sample checkout points
        sqlx::query(
            r#"
            INSERT INTO checkout_points (latitude, longitude, radius, location_name, allowed_department) 
            SELECT * FROM (VALUES
                (39.9042, 116.4074, 100, 'Office Building A', ARRAY[1, 99]),
                (39.9142, 116.4174, 150, 'Mining Site 1', ARRAY[2]),
                (39.9242, 116.4274, 200, 'Warehouse', ARRAY[5]),
                (39.9342, 116.4374, 100, 'Lab Building', ARRAY[6])
            ) AS v(latitude, longitude, radius, location_name, allowed_department)
            WHERE NOT EXISTS (SELECT 1 FROM checkout_points)
            "#
        )
        .execute(pool)
        .await?;

        log::info!("Sample data inserted successfully");
    }

    // Check if we need to insert sample admin users
    let admin_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM admin_user")
        .fetch_one(pool)
        .await?;

    if admin_count.0 == 0 {
        log::info!("No admin users found, inserting sample admin users...");
        
        // Insert sample admin users
        sqlx::query(
            r#"
            INSERT INTO admin_user (username, password, role, department) VALUES
            ('admin', 'admin123', 'admin', NULL),
            ('Office', 'Office123', 'department', 1),
            ('Mining', 'Mining123', 'department', 2),
            ('CA', 'CA123', 'department', 3),
            ('HR', 'HR123', 'department', 4),
            ('Warehouse', 'Warehouse123', 'department', 5),
            ('Lab', 'Lab123', 'department', 6),
            ('Logistics', 'Logistics123', 'department', 7),
            ('Training', 'Training123', 'department', 8),
            ('Technic', 'Technic123', 'department', 9),
            ('Hydro', 'Hydro123', 'department', 10),
            ('Washing', 'Washing123', 'department', 11),
            ('Instrument', 'Instrument123', 'department', 12),
            ('Mobile', 'Mobile123', 'department', 13),
            ('Dispatch', 'Dispatch123', 'department', 14),
            ('Beneficiation', 'Beneficiation123', 'department', 15),
            ('Enterprise', 'Enterprise123', 'department', 16),
            ('Fixed', 'Fixed123', 'department', 17),
            ('HSE', 'HSE123', 'department', 18),
            ('OfficeCamp', 'OfficeCamp123', 'department', 18),
            ('Equipment', 'Equipment123', 'department', 20),
            ('Finance', 'Finance123', 'department', 21),
            ('Medic', 'Medic123', 'department', 22),
            ('Standby', 'Standby123', 'department', 99)
            ON CONFLICT (username) DO NOTHING
            "#
        )
        .execute(pool)
        .await?;

        log::info!("Sample admin users inserted successfully");
    }
    
    Ok(())
}