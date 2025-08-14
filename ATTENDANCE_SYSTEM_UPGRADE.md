# Attendance System Upgrade Documentation

## Overview
The attendance system has been upgraded from a single daily record system to a robust session-based tracking system that supports multiple check-ins/check-outs per day.

## Key Improvements

### 1. Session-Based Tracking
- **Previous Issue**: Only stored one check-in and one check-out per day
- **Solution**: New `attendance_sessions` table tracks individual work sessions
- **Benefits**: 
  - Supports multiple work sessions per day
  - Tracks lunch breaks and site visits
  - Maintains complete audit trail

### 2. Database Schema Changes

#### New Tables
- **`attendance_sessions`**: Stores individual work sessions
  - `session_number`: Sequential number for each session in a day
  - `duration_minutes`: Auto-calculated session duration
  - `is_complete`: Indicates if session has both check-in and check-out
  - Location tracking for both check-in and check-out

- **`attendance_summary`** (renamed from `attendance_info`): Daily aggregated data
  - `first_checkin_time`: First check-in of the day
  - `last_checkout_time`: Last check-out of the day
  - `total_work_minutes`: Sum of all session durations
  - `total_sessions`: Number of sessions in the day
  - `is_complete`: All sessions properly closed

### 3. Migration Strategy
The migration file (`002_attendance_sessions.sql`) includes:
- Automatic data migration from existing `checkins` table
- Triggers for automatic duration calculation
- Triggers for automatic summary updates
- Backward compatibility fields

### 4. Business Logic Updates

#### Session Management Algorithm
```
1. Receives check-ins/outs from mobile app
2. Groups by date and sorts chronologically
3. For each check-in/out:
   - IN action: Opens new session or flags duplicate
   - OUT action: Closes current session or creates partial session
4. Auto-calculates durations and updates summary
```

#### Edge Case Handling
- Multiple INs without OUT: Uses last IN
- OUT without IN: Creates partial session
- Day boundary crossings: Handled by date grouping

### 5. New API Endpoints

#### `/api/sessions/daily` (POST)
Get detailed session information for a specific day
```json
Request:
{
  "user_id": "user_001",
  "passkey": "...",
  "date": "2024-01-15"
}

Response:
{
  "success": true,
  "message": "Daily sessions retrieved",
  "data": {
    "date": "2024-01-15",
    "sessions": [...],
    "summary": {...}
  }
}
```

### 6. Enhanced Monthly Statistics
The monthly stats now include:
- `total_work_minutes`: Total work time per day
- `total_sessions`: Number of sessions per day
- Better accuracy for late/early leave calculations

## Database Migration

To apply the migration:
```bash
psql -U your_user -d your_database -f migrations/002_attendance_sessions.sql
```

## Testing Recommendations

1. **Test Multiple Sessions**: Verify multiple check-ins/outs per day work correctly
2. **Test Edge Cases**: 
   - Check-in without check-out
   - Check-out without check-in
   - Multiple consecutive check-ins
3. **Verify Data Migration**: Ensure historical data migrated correctly
4. **Performance Testing**: Test with large datasets

## Rollback Plan

If rollback is needed:
```sql
-- Rename attendance_summary back to attendance_info
ALTER TABLE attendance_summary RENAME TO attendance_info;

-- Drop new columns
ALTER TABLE attendance_info 
  DROP COLUMN IF EXISTS first_checkin_time,
  DROP COLUMN IF EXISTS last_checkout_time,
  DROP COLUMN IF EXISTS total_work_minutes,
  DROP COLUMN IF EXISTS total_break_minutes,
  DROP COLUMN IF EXISTS total_sessions,
  DROP COLUMN IF EXISTS is_complete,
  DROP COLUMN IF EXISTS updated_at;

-- Drop attendance_sessions table
DROP TABLE IF EXISTS attendance_sessions CASCADE;

-- Remove triggers and functions
DROP TRIGGER IF EXISTS update_session_duration_trigger ON attendance_sessions;
DROP TRIGGER IF EXISTS update_summary_on_session_change ON attendance_sessions;
DROP FUNCTION IF EXISTS update_session_duration();
DROP FUNCTION IF EXISTS update_attendance_summary();
```

## Future Enhancements

1. **Break Time Calculation**: Calculate break time between sessions
2. **Overtime Tracking**: Track overtime hours automatically
3. **Location Analytics**: Analyze time spent at different locations
4. **Session Patterns**: Identify and report unusual attendance patterns
5. **Flexible Work Hours**: Support different work schedules per department

## API Backward Compatibility

The system maintains backward compatibility:
- `attendance_info` type alias points to `AttendanceSummary`
- `checkin_time` and `checkout_time` fields retained in summary
- Existing endpoints continue to work without modification

## Performance Considerations

- Indexes added on frequently queried columns
- Triggers minimize application-level calculations
- Summary table provides pre-aggregated data for reports

## Security Notes

- All endpoints require user authentication via passkey
- Session data is user-specific and access-controlled
- Location data is optional and privacy-conscious