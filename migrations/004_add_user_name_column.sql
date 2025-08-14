-- Migration to add user_name column to user_info table
-- This will store Chinese names for display while keeping user_id as unique identifier

ALTER TABLE user_info ADD COLUMN IF NOT EXISTS user_name VARCHAR(255);

-- Update existing users with sample Chinese names based on their user_id
UPDATE user_info SET user_name = CASE 
    WHEN user_id = 'user_001' THEN '张三'
    WHEN user_id = 'user_002' THEN '李四'
    WHEN user_id = 'user_003' THEN '王五'
    ELSE user_id  -- For any other users, use user_id as fallback
END
WHERE user_name IS NULL;