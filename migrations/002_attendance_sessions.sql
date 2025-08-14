-- Migration: Enhance attendance tracking to support multiple daily sessions
-- This migration creates a session-based attendance tracking system

-- Create attendance_sessions table for tracking individual work sessions
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

-- Rename existing attendance_info to attendance_summary
ALTER TABLE attendance_info RENAME TO attendance_summary;

-- Add new columns to attendance_summary for enhanced tracking
ALTER TABLE attendance_summary 
    ADD COLUMN IF NOT EXISTS first_checkin_time TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS last_checkout_time TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS total_work_minutes INTEGER DEFAULT 0,
    ADD COLUMN IF NOT EXISTS total_break_minutes INTEGER DEFAULT 0,
    ADD COLUMN IF NOT EXISTS total_sessions INTEGER DEFAULT 0,
    ADD COLUMN IF NOT EXISTS is_complete BOOLEAN DEFAULT false,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Update existing columns to maintain backward compatibility
ALTER TABLE attendance_summary 
    ALTER COLUMN checkin_time SET DATA TYPE TIMESTAMP WITH TIME ZONE,
    ALTER COLUMN checkout_time SET DATA TYPE TIMESTAMP WITH TIME ZONE;

-- Create indexes for better performance
CREATE INDEX idx_attendance_sessions_user_date ON attendance_sessions(user_id, date);
CREATE INDEX idx_attendance_sessions_date ON attendance_sessions(date);
CREATE INDEX idx_attendance_sessions_user_id ON attendance_sessions(user_id);
CREATE INDEX idx_attendance_summary_updated ON attendance_summary(updated_at);

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
CREATE TRIGGER update_summary_on_session_change
    AFTER INSERT OR UPDATE OR DELETE ON attendance_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_attendance_summary();

-- Migrate existing data from checkins table to attendance_sessions
-- This creates sessions based on the checkins history
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
    -- Match check-ins with their corresponding check-outs
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
    -- Number the sessions for each user and day
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
ON CONFLICT (user_id, date, session_number) DO NOTHING;

-- Add comment to tables
COMMENT ON TABLE attendance_sessions IS 'Tracks individual work sessions with check-in and check-out times';
COMMENT ON TABLE attendance_summary IS 'Daily attendance summary with aggregated statistics';
COMMENT ON COLUMN attendance_sessions.session_number IS 'Sequential number of the session within a day (1, 2, 3...)';
COMMENT ON COLUMN attendance_sessions.is_complete IS 'True if session has both check-in and check-out';
COMMENT ON COLUMN attendance_summary.total_work_minutes IS 'Sum of all completed session durations for the day';
COMMENT ON COLUMN attendance_summary.is_complete IS 'True if all sessions for the day are complete';