-- Test script for midnight crossing session fix
-- This creates test data to verify the midnight crossing solution works

-- Clear existing test data (optional - be careful in production!)
-- DELETE FROM attendance_sessions WHERE user_id IN ('TEST001', 'TEST002');
-- DELETE FROM checkins WHERE user_id IN ('TEST001', 'TEST002');

-- Insert test user if not exists
INSERT INTO user_info (user_id, user_name, department, department_name, passkey) VALUES
('TEST001', 'Night Shift Worker', 2, 'Mining', 'test_midnight_001'),
('TEST002', 'Regular Day Worker', 1, 'Office', 'test_midnight_002')
ON CONFLICT (user_id) DO NOTHING;

-- Test Case 1: Normal same-day session (should work as before)
INSERT INTO checkins (user_id, action, created_at, latitude, longitude, is_synced) VALUES
('TEST002', 'IN', '2024-12-10 08:00:00+10', 5.56919, 145.22116, 1),
('TEST002', 'OUT', '2024-12-10 17:00:00+10', 5.56919, 145.22116, 1);

-- Test Case 2: Cross-midnight session (currently broken, should be fixed)
INSERT INTO checkins (user_id, action, created_at, latitude, longitude, is_synced) VALUES
('TEST001', 'IN', '2024-12-10 23:30:00+10', 5.57655, 145.20210, 1),
('TEST001', 'OUT', '2024-12-11 06:30:00+10', 5.57655, 145.20210, 1);

-- Test Case 3: Multiple cross-midnight sessions
INSERT INTO checkins (user_id, action, created_at, latitude, longitude, is_synced) VALUES
('TEST001', 'IN', '2024-12-11 22:00:00+10', 5.57655, 145.20210, 1),
('TEST001', 'OUT', '2024-12-12 05:00:00+10', 5.57655, 145.20210, 1);

-- Test Case 4: Very short cross-midnight session (2 minutes)
INSERT INTO checkins (user_id, action, created_at, latitude, longitude, is_synced) VALUES
('TEST001', 'IN', '2024-12-12 23:59:00+10', 5.57655, 145.20210, 1),
('TEST001', 'OUT', '2024-12-13 00:01:00+10', 5.57655, 145.20210, 1);

-- Now run the fixed session processing
-- (This will use the updated populate_sessions_from_checkins.sql logic)

-- Verification queries - run these AFTER processing sessions

-- 1. Check if cross-midnight sessions were created properly
SELECT 
    user_id,
    date,
    session_number,
    checkin_time AT TIME ZONE 'UTC+10' as checkin_local,
    checkout_time AT TIME ZONE 'UTC+10' as checkout_local,
    duration_minutes,
    is_complete,
    CASE 
        WHEN checkout_time::date > checkin_time::date THEN 'Cross-midnight'
        ELSE 'Same-day'
    END as session_type
FROM attendance_sessions 
WHERE user_id IN ('TEST001', 'TEST002')
ORDER BY user_id, date, session_number;

-- 2. Expected results:
-- TEST002: 1 same-day session (8 hours, 480 minutes)
-- TEST001: 3 cross-midnight sessions 
--   - Dec 10: 23:30 → Dec 11: 06:30 (7 hours, 420 minutes)
--   - Dec 11: 22:00 → Dec 12: 05:00 (7 hours, 420 minutes)  
--   - Dec 12: 23:59 → Dec 13: 00:01 (2 minutes)

-- 3. Check for any incomplete sessions (should be none with the fix)
SELECT 
    'Incomplete sessions after fix' as description,
    COUNT(*) as count
FROM attendance_sessions 
WHERE user_id IN ('TEST001', 'TEST002') 
    AND is_complete = false;

-- 4. Summary of test results
SELECT 
    user_id,
    COUNT(*) as total_sessions,
    COUNT(CASE WHEN is_complete THEN 1 END) as complete_sessions,
    COUNT(CASE WHEN checkout_time::date > checkin_time::date THEN 1 END) as cross_midnight_sessions,
    SUM(duration_minutes) as total_minutes,
    AVG(duration_minutes) as avg_session_minutes
FROM attendance_sessions 
WHERE user_id IN ('TEST001', 'TEST002')
GROUP BY user_id
ORDER BY user_id;

-- Clean up test data (uncomment if needed)
-- DELETE FROM attendance_sessions WHERE user_id IN ('TEST001', 'TEST002');
-- DELETE FROM checkins WHERE user_id IN ('TEST001', 'TEST002');
-- DELETE FROM user_info WHERE user_id IN ('TEST001', 'TEST002');
