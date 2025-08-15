-- Improved script to populate attendance_sessions from checkins data
-- This script provides safer options for different scenarios

-- USAGE SCENARIOS:
-- 1. Fresh database: Run as-is to process all checkins
-- 2. Specific date range: Uncomment and modify the date filters
-- 3. Specific users: Uncomment and modify the user filters  
-- 4. Problem diagnosis: Use the verification queries at the end

-- WARNING: This script processes existing data and may overwrite attendance_sessions
-- Always backup your data before running in production!

-- Option 1: Process ALL checkins (DANGEROUS in production with existing data)
-- Uncomment the block below ONLY for fresh databases or full reprocessing

/*
INSERT INTO attendance_sessions (
    user_id, date, session_number, checkin_time, checkout_time,
    checkin_latitude, checkin_longitude, checkout_latitude, checkout_longitude,
    is_complete, duration_minutes
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
        user_id, date, checkin_time, checkout_time,
        checkin_lat, checkin_lon, checkout_lat, checkout_lon,
        ROW_NUMBER() OVER (PARTITION BY user_id, date ORDER BY checkin_time) as session_num
    FROM checkin_pairs
)
SELECT 
    user_id, date, session_num as session_number, checkin_time, checkout_time,
    checkin_lat as checkin_latitude, checkin_lon as checkin_longitude,
    checkout_lat as checkout_latitude, checkout_lon as checkout_longitude,
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
*/

-- Option 2: Process ONLY specific date range (SAFER)
-- Modify the dates below for your needs
INSERT INTO attendance_sessions (
    user_id, date, session_number, checkin_time, checkout_time,
    checkin_latitude, checkin_longitude, checkout_latitude, checkout_longitude,
    is_complete, duration_minutes
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
        AND ci.created_at >= '2024-12-01'::date  -- Modify start date
        AND ci.created_at < '2024-12-31'::date   -- Modify end date
),
numbered_sessions AS (
    SELECT 
        user_id, date, checkin_time, checkout_time,
        checkin_lat, checkin_lon, checkout_lat, checkout_lon,
        ROW_NUMBER() OVER (PARTITION BY user_id, date ORDER BY checkin_time) as session_num
    FROM checkin_pairs
)
SELECT 
    user_id, date, session_num as session_number, checkin_time, checkout_time,
    checkin_lat as checkin_latitude, checkin_lon as checkin_longitude,
    checkout_lat as checkout_latitude, checkout_lon as checkout_longitude,
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

-- Option 3: Process ONLY specific users (uncomment and modify as needed)
/*
-- Example: Process only specific user IDs
-- Add WHERE clause like: AND ci.user_id IN ('FA0ED004', '6506C554', 'FA007DA4')
*/

-- VERIFICATION QUERIES - Run these to check the results

-- Check session counts by user and date
SELECT 
    user_id,
    date,
    COUNT(*) as session_count,
    SUM(CASE WHEN is_complete THEN 1 ELSE 0 END) as complete_sessions,
    SUM(duration_minutes) as total_minutes
FROM attendance_sessions 
WHERE date >= '2024-12-01' AND date <= '2024-12-31'  -- Adjust date range
GROUP BY user_id, date 
ORDER BY user_id, date;

-- Check for potential data issues
SELECT 
    'Orphaned IN checkins' as issue_type,
    COUNT(*) as count
FROM checkins ci
WHERE ci.action = 'IN' 
    AND NOT EXISTS (
        SELECT 1 FROM checkins co 
        WHERE co.user_id = ci.user_id 
            AND co.action = 'OUT' 
            AND co.created_at > ci.created_at
            AND co.created_at::date = ci.created_at::date
    )
    AND ci.created_at >= '2024-12-01'

UNION ALL

SELECT 
    'Checkins without sessions' as issue_type,
    COUNT(DISTINCT ci.user_id || ci.created_at::text) as count
FROM checkins ci
WHERE ci.created_at >= '2024-12-01'
    AND NOT EXISTS (
        SELECT 1 FROM attendance_sessions s
        WHERE s.user_id = ci.user_id 
            AND s.date = ci.created_at::date
    );

-- Summary comparison
SELECT 
    'Total checkins' as metric,
    COUNT(*) as count
FROM checkins 
WHERE created_at >= '2024-12-01'

UNION ALL

SELECT 
    'Total sessions created' as metric,
    COUNT(*) as count
FROM attendance_sessions 
WHERE date >= '2024-12-01'

UNION ALL

SELECT 
    'Total summaries updated' as metric,
    COUNT(*) as count
FROM attendance_summary 
WHERE date >= '2024-12-01';

-- Note: The attendance_summary table will be automatically updated by the existing trigger
-- 'update_summary_on_session_change' which runs AFTER INSERT OR UPDATE OR DELETE ON attendance_sessions

