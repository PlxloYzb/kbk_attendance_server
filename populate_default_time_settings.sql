-- Populate default time settings for all existing users
-- This script can be run manually if the user_time_settings table is empty

INSERT INTO user_time_settings (user_id, on_duty_time, off_duty_time)
SELECT 
    user_id,
    '07:30:00'::time as on_duty_time,
    '17:00:00'::time as off_duty_time
FROM user_info
WHERE user_id NOT IN (SELECT user_id FROM user_time_settings)
ON CONFLICT (user_id) DO NOTHING;

-- Verify the population
SELECT 
    COUNT(*) as total_users,
    (SELECT COUNT(*) FROM user_time_settings) as users_with_time_settings
FROM user_info;

-- Show sample of populated data
SELECT 
    ui.user_id,
    ui.user_name,
    ui.department,
    uts.on_duty_time,
    uts.off_duty_time
FROM user_info ui
LEFT JOIN user_time_settings uts ON ui.user_id = uts.user_id
ORDER BY ui.department, ui.user_id
LIMIT 10;
