-- Initial schema with session-based attendance tracking
-- This script creates all tables needed for the KBK Attendance System

-- Create checkins table (raw check-in/out records)
CREATE TABLE IF NOT EXISTS checkins (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    action VARCHAR(10) NOT NULL CHECK (action IN ('IN', 'OUT')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    is_synced INTEGER DEFAULT 0 CHECK (is_synced IN (0, 1))
);

-- Create attendance_sessions table (individual work sessions)
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
);

-- Create attendance_summary table (daily aggregated data)
CREATE TABLE IF NOT EXISTS attendance_summary (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    date DATE NOT NULL,
    checkin_time TIMESTAMP WITH TIME ZONE, -- Backward compatibility
    checkout_time TIMESTAMP WITH TIME ZONE, -- Backward compatibility
    first_checkin_time TIMESTAMP WITH TIME ZONE,
    last_checkout_time TIMESTAMP WITH TIME ZONE,
    total_work_minutes INTEGER DEFAULT 0,
    total_break_minutes INTEGER DEFAULT 0,
    total_sessions INTEGER DEFAULT 0,
    is_complete BOOLEAN DEFAULT false,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, date)
);

-- Create checkin_points table
CREATE TABLE IF NOT EXISTS checkin_points (
    id SERIAL PRIMARY KEY,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    radius DOUBLE PRECISION NOT NULL,
    location_name VARCHAR(255) NOT NULL,
    allowed_department INTEGER[] DEFAULT ARRAY[0]
);

-- Create checkout_points table
CREATE TABLE IF NOT EXISTS checkout_points (
    id SERIAL PRIMARY KEY,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    radius DOUBLE PRECISION NOT NULL,
    location_name VARCHAR(255) NOT NULL,
    allowed_department INTEGER[] DEFAULT ARRAY[0]
);

-- Create user_info table
CREATE TABLE IF NOT EXISTS user_info (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) UNIQUE NOT NULL,
    department INTEGER DEFAULT 99,
    department_name VARCHAR(100),
    department_code VARCHAR(10),
    passkey VARCHAR(255) NOT NULL
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_checkins_user_id ON checkins(user_id);
CREATE INDEX IF NOT EXISTS idx_checkins_created_at ON checkins(created_at);
CREATE INDEX IF NOT EXISTS idx_attendance_sessions_user_date ON attendance_sessions(user_id, date);
CREATE INDEX IF NOT EXISTS idx_attendance_sessions_date ON attendance_sessions(date);
CREATE INDEX IF NOT EXISTS idx_attendance_sessions_user_id ON attendance_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_attendance_summary_user_id ON attendance_summary(user_id);
CREATE INDEX IF NOT EXISTS idx_attendance_summary_date ON attendance_summary(date);
CREATE INDEX IF NOT EXISTS idx_attendance_summary_updated ON attendance_summary(updated_at);
CREATE INDEX IF NOT EXISTS idx_user_info_passkey ON user_info(passkey);

-- Create function to update duration when checkout_time is set
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
$$ LANGUAGE plpgsql;

-- Create trigger for automatic duration calculation
DROP TRIGGER IF EXISTS update_session_duration_trigger ON attendance_sessions;
CREATE TRIGGER update_session_duration_trigger
    BEFORE UPDATE ON attendance_sessions
    FOR EACH ROW
    WHEN (OLD.checkout_time IS DISTINCT FROM NEW.checkout_time)
    EXECUTE FUNCTION update_session_duration();

-- Create function to update attendance_summary when sessions change
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
        checkin_time,  -- Keep for backward compatibility
        checkout_time, -- Keep for backward compatibility
        total_work_minutes, 
        total_sessions, 
        is_complete,
        updated_at
    ) VALUES (
        COALESCE(NEW.user_id, OLD.user_id),
        COALESCE(NEW.date, OLD.date),
        v_first_checkin,
        v_last_checkout,
        v_first_checkin,  -- Set for backward compatibility
        v_last_checkout,  -- Set for backward compatibility
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
$$ LANGUAGE plpgsql;

-- Create trigger to update summary when sessions change
DROP TRIGGER IF EXISTS update_summary_on_session_change ON attendance_sessions;
CREATE TRIGGER update_summary_on_session_change
    AFTER INSERT OR UPDATE OR DELETE ON attendance_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_attendance_summary();

-- Add comments to tables
COMMENT ON TABLE attendance_sessions IS 'Tracks individual work sessions with check-in and check-out times';
COMMENT ON TABLE attendance_summary IS 'Daily attendance summary with aggregated statistics';
COMMENT ON COLUMN attendance_sessions.session_number IS 'Sequential number of the session within a day (1, 2, 3...)';
COMMENT ON COLUMN attendance_sessions.is_complete IS 'True if session has both check-in and check-out';
COMMENT ON COLUMN attendance_summary.total_work_minutes IS 'Sum of all completed session durations for the day';
COMMENT ON COLUMN attendance_summary.is_complete IS 'True if all sessions for the day are complete';

-- Insert sample checkin points
INSERT INTO checkin_points (latitude, longitude, radius, location_name, allowed_department) 
SELECT * FROM (VALUES
    (39.9042, 116.4074, 100, 'Office Building A', ARRAY[1, 99]),
    (39.9142, 116.4174, 150, 'Mining Site 1', ARRAY[2]),
    (39.9242, 116.4274, 200, 'Warehouse', ARRAY[5]),
    (39.9342, 116.4374, 100, 'Lab Building', ARRAY[6])
) AS v(latitude, longitude, radius, location_name, allowed_department)
WHERE NOT EXISTS (SELECT 1 FROM checkin_points);

-- Insert sample checkout points  
INSERT INTO checkout_points (latitude, longitude, radius, location_name, allowed_department) 
SELECT * FROM (VALUES
    (39.9042, 116.4074, 100, 'Office Building A', ARRAY[1, 99]),
    (39.9142, 116.4174, 150, 'Mining Site 1', ARRAY[2]),
    (39.9242, 116.4274, 200, 'Warehouse', ARRAY[5]),
    (39.9342, 116.4374, 100, 'Lab Building', ARRAY[6])
) AS v(latitude, longitude, radius, location_name, allowed_department)
WHERE NOT EXISTS (SELECT 1 FROM checkout_points);

-- Insert sample users
INSERT INTO user_info (user_id, department, department_name, department_code, passkey) VALUES
('user_001', 1, 'office', '01', 'test_passkey_001'),
('user_002', 2, 'mining', '02', 'test_passkey_002'),
('user_003', 5, 'warehouse', '05', 'test_passkey_003')
ON CONFLICT (user_id) DO NOTHING;