-- Create user_time_settings table to store custom duty times per user
CREATE TABLE user_time_settings (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(50) NOT NULL,
    on_duty_time TIME NOT NULL DEFAULT '09:00:00',
    off_duty_time TIME NOT NULL DEFAULT '18:00:00',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add unique constraint on user_id to prevent duplicates
ALTER TABLE user_time_settings ADD CONSTRAINT unique_user_time_settings UNIQUE (user_id);

-- Add foreign key constraint to ensure user exists
ALTER TABLE user_time_settings ADD CONSTRAINT fk_user_time_settings_user_id 
    FOREIGN KEY (user_id) REFERENCES user_info(user_id) ON DELETE CASCADE;

-- Create index for faster queries
CREATE INDEX idx_user_time_settings_user_id ON user_time_settings(user_id);

-- Add trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_user_time_settings_updated_at 
    BEFORE UPDATE ON user_time_settings 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
