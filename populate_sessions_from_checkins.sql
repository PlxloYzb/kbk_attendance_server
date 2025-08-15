-- Script to populate attendance_sessions and attendance_summary from checkins data
-- This should be run after inserting new checkins data to ensure the other tables are updated
-- Based on the migration logic in 002_attendance_sessions.sql

-- First, let's create sessions from the new checkins data
-- This uses the same logic as the migration script
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
ON CONFLICT (user_id, date, session_number) DO UPDATE SET
    checkin_time = EXCLUDED.checkin_time,
    checkout_time = EXCLUDED.checkout_time,
    checkin_latitude = EXCLUDED.checkin_latitude,
    checkin_longitude = EXCLUDED.checkin_longitude,
    checkout_latitude = EXCLUDED.checkout_latitude,
    checkout_longitude = EXCLUDED.checkout_longitude,
    is_complete = EXCLUDED.is_complete,
    duration_minutes = EXCLUDED.duration_minutes,
    updated_at = NOW();

-- Note: The attendance_summary table will be automatically updated by the existing trigger
-- 'update_summary_on_session_change' which runs AFTER INSERT OR UPDATE OR DELETE ON attendance_sessions

-- Optional: Verify the results
-- SELECT 'Attendance Sessions Created' as status, COUNT(*) as count FROM attendance_sessions;
-- SELECT 'Attendance Summary Updated' as status, COUNT(*) as count FROM attendance_summary;

-- Optional: Show sample results for verification
-- SELECT user_id, date, session_number, checkin_time, checkout_time, duration_minutes, is_complete 
-- FROM attendance_sessions 
-- ORDER BY user_id, date, session_number 
-- LIMIT 10;

