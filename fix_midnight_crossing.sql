-- Fix for midnight crossing sessions
-- This script addresses the issue where checkin before midnight and checkout after midnight
-- creates incomplete sessions instead of properly matched pairs

-- SOLUTION 1: Extended checkout window for existing session processing
-- Updates the populate_sessions_from_checkins.sql logic to handle cross-midnight sessions

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
    -- Match check-ins with their corresponding check-outs (INCLUDING NEXT DAY)
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
                -- FIXED: Allow checkouts on the next day too
                AND co.created_at::date IN (ci.created_at::date, ci.created_at::date + INTERVAL '1 day')
                -- Add reasonable maximum session duration (16 hours)
                AND co.created_at < ci.created_at + INTERVAL '16 hours'
            ORDER BY co.created_at 
            LIMIT 1
        ) as checkout_time,
        (
            SELECT co.latitude 
            FROM checkins co 
            WHERE co.user_id = ci.user_id 
                AND co.action = 'OUT' 
                AND co.created_at > ci.created_at
                AND co.created_at::date IN (ci.created_at::date, ci.created_at::date + INTERVAL '1 day')
                AND co.created_at < ci.created_at + INTERVAL '16 hours'
            ORDER BY co.created_at 
            LIMIT 1
        ) as checkout_lat,
        (
            SELECT co.longitude 
            FROM checkins co 
            WHERE co.user_id = ci.user_id 
                AND co.action = 'OUT' 
                AND co.created_at > ci.created_at
                AND co.created_at::date IN (ci.created_at::date, ci.created_at::date + INTERVAL '1 day')
                AND co.created_at < ci.created_at + INTERVAL '16 hours'
            ORDER BY co.created_at 
            LIMIT 1
        ) as checkout_lon
    FROM checkins ci
    WHERE ci.action = 'IN'
),
numbered_sessions AS (
    -- Number the sessions for each user and day (assign session to checkin date)
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

-- SOLUTION 2: Identify and fix existing broken cross-midnight sessions
-- Find potential cross-midnight sessions that are currently incomplete

WITH potential_cross_midnight AS (
    -- Find incomplete checkins that might have checkouts on the next day
    SELECT DISTINCT
        ci.user_id,
        ci.created_at as checkin_time,
        ci.created_at::date as checkin_date,
        co.created_at as checkout_time,
        co.created_at::date as checkout_date
    FROM checkins ci
    JOIN checkins co ON (
        ci.user_id = co.user_id 
        AND co.action = 'OUT'
        AND co.created_at > ci.created_at
        AND co.created_at::date = ci.created_at::date + INTERVAL '1 day'
        AND co.created_at < ci.created_at + INTERVAL '16 hours'
    )
    WHERE ci.action = 'IN'
    -- Only for checkins that don't have a session yet or have incomplete sessions
    AND NOT EXISTS (
        SELECT 1 FROM attendance_sessions s 
        WHERE s.user_id = ci.user_id 
            AND s.date = ci.created_at::date
            AND s.checkin_time = ci.created_at
            AND s.checkout_time IS NOT NULL
    )
),
cross_midnight_stats AS (
    SELECT 
        'Cross-midnight sessions found' as description,
        COUNT(*) as count
    FROM potential_cross_midnight
)
SELECT * FROM cross_midnight_stats;

-- VERIFICATION QUERIES
-- Run these to check for midnight crossing issues

-- 1. Find all incomplete sessions that might be cross-midnight
SELECT 
    'Potentially broken cross-midnight sessions' as issue_type,
    COUNT(*) as count
FROM attendance_sessions s
WHERE s.checkout_time IS NULL
    AND s.checkin_time::time > '22:00:00'  -- Late night checkins
    AND EXISTS (
        SELECT 1 FROM checkins co 
        WHERE co.user_id = s.user_id 
            AND co.action = 'OUT'
            AND co.created_at::date = s.date + INTERVAL '1 day'
            AND co.created_at < s.checkin_time + INTERVAL '16 hours'
    );

-- 2. Find orphaned checkouts that might belong to previous day checkins
SELECT 
    'Orphaned early-morning checkouts' as issue_type,
    COUNT(*) as count  
FROM checkins co
WHERE co.action = 'OUT'
    AND co.created_at::time < '08:00:00'  -- Early morning checkouts
    AND NOT EXISTS (
        SELECT 1 FROM attendance_sessions s
        WHERE s.user_id = co.user_id
            AND s.date = co.created_at::date
            AND s.checkout_time = co.created_at
    )
    AND EXISTS (
        SELECT 1 FROM checkins ci
        WHERE ci.user_id = co.user_id
            AND ci.action = 'IN' 
            AND ci.created_at::date = co.created_at::date - INTERVAL '1 day'
            AND ci.created_at::time > '22:00:00'
    );

-- 3. Summary of session duration patterns  
SELECT 
    CASE 
        WHEN duration_minutes IS NULL THEN 'Incomplete'
        WHEN duration_minutes > 960 THEN 'Very Long (16+ hours)'
        WHEN duration_minutes > 720 THEN 'Long (12-16 hours)' 
        WHEN duration_minutes > 480 THEN 'Normal (8-12 hours)'
        WHEN duration_minutes > 240 THEN 'Short (4-8 hours)'
        ELSE 'Very Short (< 4 hours)'
    END as duration_category,
    COUNT(*) as session_count,
    AVG(duration_minutes) as avg_minutes
FROM attendance_sessions 
WHERE date >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY 
    CASE 
        WHEN duration_minutes IS NULL THEN 'Incomplete'
        WHEN duration_minutes > 960 THEN 'Very Long (16+ hours)'
        WHEN duration_minutes > 720 THEN 'Long (12-16 hours)' 
        WHEN duration_minutes > 480 THEN 'Normal (8-12 hours)'
        WHEN duration_minutes > 240 THEN 'Short (4-8 hours)'
        ELSE 'Very Short (< 4 hours)'
    END
ORDER BY session_count DESC;

-- 4. Users with frequent late-night checkins (potential night shift workers)
SELECT 
    user_id,
    COUNT(*) as late_checkins,
    COUNT(CASE WHEN checkout_time IS NULL THEN 1 END) as incomplete_sessions,
    AVG(duration_minutes) as avg_duration
FROM attendance_sessions s
WHERE s.checkin_time::time > '20:00:00'  -- After 8 PM
    AND date >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY user_id
HAVING COUNT(*) > 2  -- More than 2 late checkins
ORDER BY incomplete_sessions DESC, late_checkins DESC;

-- Note: After running this analysis, you can apply the fixed session logic
-- to resolve the cross-midnight issues in your attendance data
