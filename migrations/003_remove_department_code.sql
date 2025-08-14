-- Migration to remove department_code column from user_info table
-- Since department_code is redundant (just a zero-padded string version of department integer)

ALTER TABLE user_info DROP COLUMN IF EXISTS department_code;