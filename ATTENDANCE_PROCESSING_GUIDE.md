# Attendance Data Processing Guide

## Overview
This guide explains when and how to use the attendance processing scripts safely.

## Data Flow Architecture

```
checkins → attendance_sessions → attendance_summary
    ↑              ↑                    ↑
Direct SQL    Manual Script        Auto Trigger
Admin UI      /sync endpoint      (on sessions change)
```

## Processing Methods

### 1. ✅ Automatic (Recommended)
- **App `/sync` endpoint**: Processes new checkins into sessions automatically
- **Fixed Admin UI**: Now properly creates sessions when adding checkins
- **Database triggers**: Auto-update attendance_summary when sessions change

### 2. ⚠️ Manual Script Processing

#### Safe Scenarios:
- **Fresh database** with checkins but no sessions
- **Initial data migration** from old systems
- **Specific date range** processing for known problematic data
- **Data recovery** after attendance_sessions corruption

#### Dangerous Scenarios:
- **Production system** with existing sessions data
- **Live operations** where users are actively checking in/out
- **Unknown data state** - always verify first

## Script Usage Guidelines

### Before Running ANY Script:
1. **Backup your database**
2. **Check existing data state**:
   ```sql
   SELECT COUNT(*) FROM checkins;
   SELECT COUNT(*) FROM attendance_sessions;
   SELECT COUNT(*) FROM attendance_summary;
   ```

### Choose the Right Script:

#### For Fresh Database (No Sessions):
```bash
# Use the basic script
\i populate_sessions_from_checkins.sql
```

#### For Specific Date Range:
```bash
# Use the improved script with date filters
\i populate_sessions_from_checkins_improved.sql
```

#### For Problem Diagnosis:
1. Run verification queries first
2. Process only problematic date ranges
3. Verify results with summary queries

## Common Issues & Solutions

### Issue: "Duplicate sessions being created"
**Cause**: Running script on data that already has sessions
**Solution**: Use date-filtered processing or clear sessions first

### Issue: "Sessions don't match checkins"
**Cause**: Complex checkin patterns (multiple IN without OUT, etc.)
**Solution**: 
1. Check for orphaned checkins
2. Manual data cleanup before processing
3. Use verification queries to identify patterns

### Issue: "attendance_summary not updating"
**Cause**: Database triggers not functioning
**Solution**:
1. Check trigger exists: `\d+ attendance_sessions`
2. Manual summary refresh if needed

## Best Practices

### 1. Development/Testing:
- ✅ Use scripts freely for test data
- ✅ Experiment with different scenarios
- ✅ Verify results with query tools

### 2. Production:
- ❌ Never run full reprocessing during business hours
- ✅ Always backup before running scripts
- ✅ Process small date ranges incrementally
- ✅ Verify results before committing changes

### 3. Data Migration:
- ✅ Process historical data in batches
- ✅ Verify each batch before proceeding
- ✅ Keep old system data until migration is confirmed

## Verification Checklist

After running any processing script:

- [ ] Check session counts match expected patterns
- [ ] Verify attendance_summary was updated
- [ ] Test that admin UI still works correctly
- [ ] Run sample queries to ensure data integrity
- [ ] Check for any orphaned or incomplete records

## When Scripts Are NOT Needed

- ✅ **Normal app usage**: `/sync` endpoint handles everything
- ✅ **Admin checkin creation**: Fixed backend now processes properly
- ✅ **Regular operations**: Triggers handle summary updates automatically

## Emergency Recovery

If something goes wrong:

1. **Stop all attendance operations**
2. **Restore from backup**
3. **Identify the root cause**
4. **Process problematic data only**
5. **Verify system integrity before resuming**

Remember: The goal is reliable, automatic processing. Manual scripts should be the exception, not the norm!

