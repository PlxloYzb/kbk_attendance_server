// Statistics functionality for department users
const stats = {
    async loadDepartmentStats() {
        try {
            const response = await api.getDepartmentStats();
            if (response.success) {
                this.displayDepartmentStats(response.data);
            } else {
                document.getElementById('stats-content').innerHTML = '<p class="error">Failed to load statistics</p>';
            }
        } catch (error) {
            document.getElementById('stats-content').innerHTML = '<p class="error">Error loading statistics</p>';
        }
    },
    
    displayDepartmentStats(data) {
        const statsContent = document.getElementById('stats-content');
        
        if (!data.departments || data.departments.length === 0) {
            statsContent.innerHTML = '<p>No statistics available</p>';
            return;
        }
        
        let html = '';
        
        data.departments.forEach(dept => {
            html += `
                <div class="dept-stats">
                    <h3>Department ${dept.department} - ${dept.department_name || 'N/A'}</h3>
                    <div class="stats-summary">
                        <div class="stat-item">
                            <span class="stat-label">Total Users:</span>
                            <span class="stat-value">${dept.user_count}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Total Attendance Days:</span>
                            <span class="stat-value">${dept.total_attendance_days}</span>
                        </div>
                        <div class="stat-item">
                            <span class="stat-label">Average Work Hours:</span>
                            <span class="stat-value">${dept.avg_work_hours.toFixed(2)}</span>
                        </div>
                    </div>
                    
                    <h4>User Details:</h4>
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>User ID</th>
                                <th>Total Days</th>
                                <th>Total Hours</th>
                                <th>Last Checkin</th>
                            </tr>
                        </thead>
                        <tbody>
            `;
            
            dept.users.forEach(user => {
                const lastCheckin = user.last_checkin ? 
                    new Date(user.last_checkin).toLocaleString() : 'Never';
                
                html += `
                    <tr>
                        <td>${user.user_id}</td>
                        <td>${user.total_days}</td>
                        <td>${user.total_hours.toFixed(2)}</td>
                        <td>${lastCheckin}</td>
                    </tr>
                `;
            });
            
            html += `
                        </tbody>
                    </table>
                </div>
            `;
        });
        
        statsContent.innerHTML = html;
    }
};