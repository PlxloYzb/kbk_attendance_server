// Statistics functionality for department users
const stats = {
    async loadDepartmentStats() {
        const content = document.getElementById('stats-content');
        this.setupStatsFilters(content);
        await this.loadFilteredStats();
    },

    setupStatsFilters(content) {
        const now = new Date();
        const currentYear = now.getFullYear();
        const currentMonth = now.getMonth() + 1;
        
        const filtersHtml = `
            <div class="stats-filters">
                <div class="filter-row">
                    <div class="filter-group">
                        <label for="stats-view-type">View:</label>
                        <select id="stats-view-type">
                            <option value="month" selected>Monthly</option>
                            <option value="year">Yearly</option>
                        </select>
                    </div>
                    
                    <div class="filter-group">
                        <label for="stats-month">Month:</label>
                        <select id="stats-month">
                            <option value="1" ${currentMonth === 1 ? 'selected' : ''}>January</option>
                            <option value="2" ${currentMonth === 2 ? 'selected' : ''}>February</option>
                            <option value="3" ${currentMonth === 3 ? 'selected' : ''}>March</option>
                            <option value="4" ${currentMonth === 4 ? 'selected' : ''}>April</option>
                            <option value="5" ${currentMonth === 5 ? 'selected' : ''}>May</option>
                            <option value="6" ${currentMonth === 6 ? 'selected' : ''}>June</option>
                            <option value="7" ${currentMonth === 7 ? 'selected' : ''}>July</option>
                            <option value="8" ${currentMonth === 8 ? 'selected' : ''}>August</option>
                            <option value="9" ${currentMonth === 9 ? 'selected' : ''}>September</option>
                            <option value="10" ${currentMonth === 10 ? 'selected' : ''}>October</option>
                            <option value="11" ${currentMonth === 11 ? 'selected' : ''}>November</option>
                            <option value="12" ${currentMonth === 12 ? 'selected' : ''}>December</option>
                        </select>
                    </div>
                    
                    <div class="filter-group">
                        <label for="stats-year">Year:</label>
                        <select id="stats-year">
                            <option value="${currentYear - 1}">${currentYear - 1}</option>
                            <option value="${currentYear}" selected>${currentYear}</option>
                            <option value="${currentYear + 1}">${currentYear + 1}</option>
                        </select>
                    </div>
                    
                    <div class="filter-group">
                        <label for="stats-user-search">Search User:</label>
                        <input type="text" id="stats-user-search" placeholder="Enter user name...">
                    </div>
                    
                    <div class="filter-group">
                        <button id="apply-stats-filter" class="btn btn-primary">Apply Filters</button>
                        <button id="reset-stats-filter" class="btn btn-secondary">Reset</button>
                    </div>
                </div>
            </div>
            <div id="stats-results"></div>
        `;
        
        content.innerHTML = filtersHtml;
        
        // Add event listeners
        document.getElementById('apply-stats-filter').addEventListener('click', () => this.loadFilteredStats());
        document.getElementById('reset-stats-filter').addEventListener('click', () => this.resetStatsFilters());
        
        // Add view type change listener
        document.getElementById('stats-view-type').addEventListener('change', () => {
            this.handleViewTypeChange();
            this.loadFilteredStats();
        });
        
        // Add real-time search
        let searchTimeout;
        document.getElementById('stats-user-search').addEventListener('input', () => {
            clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => this.loadFilteredStats(), 500);
        });
        
        // Initialize view type state
        this.handleViewTypeChange();
    },

    handleViewTypeChange() {
        const viewType = document.getElementById('stats-view-type').value;
        const monthSelect = document.getElementById('stats-month');
        
        if (viewType === 'year') {
            monthSelect.disabled = true;
            monthSelect.style.opacity = '0.5';
        } else {
            monthSelect.disabled = false;
            monthSelect.style.opacity = '1';
        }
    },

    resetStatsFilters() {
        const now = new Date();
        document.getElementById('stats-view-type').value = 'month';
        document.getElementById('stats-month').value = now.getMonth() + 1;
        document.getElementById('stats-year').value = now.getFullYear();
        document.getElementById('stats-user-search').value = '';
        this.handleViewTypeChange();
        this.loadFilteredStats();
    },

    async loadFilteredStats() {
        const resultsContainer = document.getElementById('stats-results');
        resultsContainer.innerHTML = '<div class="loading">Loading statistics...</div>';
        
        try {
            const viewType = document.getElementById('stats-view-type').value;
            const params = {
                view_type: viewType,
                year: parseInt(document.getElementById('stats-year').value),
                user_name: document.getElementById('stats-user-search').value.trim() || undefined,
            };
            
            // Only include month for monthly view
            if (viewType === 'month') {
                params.month = parseInt(document.getElementById('stats-month').value);
            }
            
            const response = await api.getFilteredDepartmentStats(params);
            if (response.success) {
                this.displayFilteredStats(response.data, params);
            } else {
                resultsContainer.innerHTML = '<p class="error">Failed to load statistics</p>';
            }
        } catch (error) {
            resultsContainer.innerHTML = '<p class="error">Error loading statistics</p>';
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
                                <th>User Name</th>
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
                        <td>${user.user_name || user.user_id}</td>
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
    },

    displayFilteredStats(data, params) {
        const content = document.getElementById('stats-results');
        
        if (!data.departments || data.departments.length === 0) {
            content.innerHTML = '<p>No statistics available for the selected filters</p>';
            return;
        }

        const monthNames = ['', 'January', 'February', 'March', 'April', 'May', 'June',
                          'July', 'August', 'September', 'October', 'November', 'December'];
        
        let periodTitle;
        if (params.view_type === 'year') {
            periodTitle = `Statistics for Year ${params.year}`;
        } else {
            const selectedMonthName = monthNames[params.month];
            periodTitle = `Statistics for ${selectedMonthName} ${params.year}`;
        }

        let html = `<div class="stats-header">
                        <h3>${periodTitle}</h3>
                        ${params.user_name ? `<p>Filtered by user: "${params.user_name}"</p>` : ''}
                    </div>`;
        
        data.departments.forEach(dept => {
            html += `
                <div class="dept-stats">
                    <h4>Department ${dept.department} - ${dept.department_name || 'N/A'}</h4>
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
                    
                    <h5>User Details:</h5>
                    <div class="table-container">
                        <table class="data-table sortable-table" data-department="${dept.department}">
                            <thead>
                                <tr>
                                    <th>User Name</th>
                                    <th>User ID</th>
                                    <th class="sortable" data-column="total_days">
                                        Total Days 
                                        <span class="sort-icon">↕</span>
                                    </th>
                                    <th class="sortable" data-column="total_hours">
                                        Total Hours 
                                        <span class="sort-icon">↕</span>
                                    </th>
                                    <th>Last Checkin</th>
                                    <th>Actions</th>
                                </tr>
                            </thead>
                            <tbody>
            `;
            
            dept.users.forEach(user => {
                const lastCheckin = user.last_checkin ? 
                    new Date(user.last_checkin).toLocaleString() : 'Never';
                
                html += `
                    <tr data-user-id="${user.user_id}" data-total-days="${user.total_days}" data-total-hours="${user.total_hours}">
                        <td>${user.user_name || user.user_id}</td>
                        <td>${user.user_id}</td>
                        <td>${user.total_days}</td>
                        <td>${user.total_hours.toFixed(2)}</td>
                        <td>${lastCheckin}</td>
                        <td>
                            <button class="btn btn-small btn-primary detail-btn" 
                                    data-user-id="${user.user_id}" 
                                    data-user-name="${user.user_name || user.user_id}"
                                    data-month="${params.month || 0}"
                                    data-year="${params.year}"
                                    data-view-type="${params.view_type}">
                                Details
                            </button>
                        </td>
                    </tr>
                `;
            });
            
            html += `
                        </tbody>
                    </table>
                    </div>
                </div>
            `;
        });
        
        content.innerHTML = html;
        
        // Add sorting functionality
        this.setupTableSorting();
        
        // Add detail button functionality
        this.setupDetailButtons();
    },

    setupTableSorting() {
        document.querySelectorAll('.sortable').forEach(header => {
            header.addEventListener('click', () => {
                const column = header.getAttribute('data-column');
                const table = header.closest('table');
                const tbody = table.querySelector('tbody');
                const rows = Array.from(tbody.querySelectorAll('tr'));
                
                // Determine sort direction
                const currentDirection = header.getAttribute('data-sort-direction') || 'none';
                let newDirection = 'asc';
                if (currentDirection === 'asc') {
                    newDirection = 'desc';
                } else if (currentDirection === 'desc') {
                    newDirection = 'asc';
                }
                
                // Clear other headers' sort direction
                table.querySelectorAll('.sortable').forEach(h => {
                    h.setAttribute('data-sort-direction', 'none');
                    h.querySelector('.sort-icon').textContent = '↕';
                });
                
                // Set current header sort direction
                header.setAttribute('data-sort-direction', newDirection);
                header.querySelector('.sort-icon').textContent = newDirection === 'asc' ? '↑' : '↓';
                
                // Sort rows
                rows.sort((a, b) => {
                    const aValue = parseFloat(a.getAttribute(`data-${column.replace('_', '-')}`));
                    const bValue = parseFloat(b.getAttribute(`data-${column.replace('_', '-')}`));
                    
                    if (newDirection === 'asc') {
                        return aValue - bValue;
                    } else {
                        return bValue - aValue;
                    }
                });
                
                // Re-append sorted rows
                rows.forEach(row => tbody.appendChild(row));
            });
        });
    },

    setupDetailButtons() {
        document.querySelectorAll('.detail-btn').forEach(btn => {
            btn.addEventListener('click', async () => {
                const userId = btn.getAttribute('data-user-id');
                const userName = btn.getAttribute('data-user-name');
                const month = parseInt(btn.getAttribute('data-month'));
                const year = parseInt(btn.getAttribute('data-year'));
                const viewType = btn.getAttribute('data-view-type');
                
                if (viewType === 'year') {
                    await this.showUserYearDetail(userId, userName, year);
                } else {
                    await this.showUserDetail(userId, userName, month, year);
                }
            });
        });
    },

    async showUserDetail(userId, userName, month, year) {
        const monthNames = ['', 'January', 'February', 'March', 'April', 'May', 'June',
                          'July', 'August', 'September', 'October', 'November', 'December'];
        
        try {
            const response = await api.getUserDetail(userId, month, year);
            if (!response.success) {
                alert('Failed to load user details: ' + response.message);
                return;
            }
            
            const data = response.data;
            const records = data.records || [];
            
            let recordsHtml = '';
            if (records.length === 0) {
                recordsHtml = '<p>No attendance records found for this month.</p>';
            } else {
                recordsHtml = `
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>Date</th>
                                <th>First Check-in</th>
                                <th>Last Check-out</th>
                                <th>Work Hours</th>
                                <th>Sessions</th>
                                <th>Status</th>
                            </tr>
                        </thead>
                        <tbody>
                `;
                
                records.forEach(record => {
                    const date = new Date(record.date).toLocaleDateString();
                    const checkin = record.first_checkin ? new Date(record.first_checkin).toLocaleTimeString() : 'N/A';
                    const checkout = record.last_checkout ? new Date(record.last_checkout).toLocaleTimeString() : 'N/A';
                    const workHours = record.total_work_minutes ? (record.total_work_minutes / 60).toFixed(2) : '0.00';
                    const sessions = record.total_sessions || 0;
                    
                    let statusBadges = [];
                    if (record.is_late) statusBadges.push('<span class="badge badge-warning">Late</span>');
                    if (record.is_early_leave) statusBadges.push('<span class="badge badge-info">Early Leave</span>');
                    const status = statusBadges.length > 0 ? statusBadges.join(' ') : '<span class="badge badge-success">Normal</span>';
                    
                    recordsHtml += `
                        <tr>
                            <td>${date}</td>
                            <td>${checkin}</td>
                            <td>${checkout}</td>
                            <td>${workHours}</td>
                            <td>${sessions}</td>
                            <td>${status}</td>
                        </tr>
                    `;
                });
                
                recordsHtml += '</tbody></table>';
            }
            
            // Create modal if it doesn't exist for department page
            if (!document.getElementById('modal')) {
                const modalHtml = `
                    <div id="modal" class="modal">
                        <div class="modal-content">
                            <div id="modal-body"></div>
                        </div>
                    </div>
                `;
                document.body.insertAdjacentHTML('beforeend', modalHtml);
            }
            
            const modalContent = `
                <div class="modal-header">
                    <h3>Attendance Details - ${userName}</h3>
                    <span class="close">&times;</span>
                </div>
                <div class="modal-body">
                    <div class="detail-summary">
                        <h4>${monthNames[month]} ${year} Summary</h4>
                        <div class="summary-stats">
                            <div class="stat-item">
                                <span class="stat-label">Total Days:</span>
                                <span class="stat-value">${data.total_days}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Total Hours:</span>
                                <span class="stat-value">${data.total_hours.toFixed(2)}</span>
                            </div>
                        </div>
                    </div>
                    <div class="detail-records">
                        <h4>Daily Records</h4>
                        ${recordsHtml}
                    </div>
                </div>
            `;
            
            document.getElementById('modal-body').innerHTML = modalContent;
            document.getElementById('modal').style.display = 'block';
            
            // Add close functionality
            const closeBtn = document.querySelector('.modal .close');
            if (closeBtn) {
                closeBtn.onclick = () => {
                    document.getElementById('modal').style.display = 'none';
                };
            }
            
            // Close when clicking outside
            window.onclick = (event) => {
                const modal = document.getElementById('modal');
                if (event.target === modal) {
                    modal.style.display = 'none';
                }
            };
            
        } catch (error) {
            alert('Error loading user details: ' + error.message);
        }
    },

    async showUserYearDetail(userId, userName, year) {
        try {
            // Get data for all 12 months
            const monthlyData = [];
            const monthNames = ['', 'January', 'February', 'March', 'April', 'May', 'June',
                              'July', 'August', 'September', 'October', 'November', 'December'];
            
            for (let month = 1; month <= 12; month++) {
                try {
                    const response = await api.getUserDetail(userId, month, year);
                    if (response.success) {
                        monthlyData.push({
                            month: month,
                            monthName: monthNames[month],
                            data: response.data
                        });
                    }
                } catch (e) {
                    // Continue if one month fails
                    console.warn(`Failed to load data for ${monthNames[month]} ${year}:`, e);
                }
            }
            
            // Calculate yearly totals
            let totalYearDays = 0;
            let totalYearHours = 0;
            
            monthlyData.forEach(monthData => {
                totalYearDays += monthData.data.total_days;
                totalYearHours += monthData.data.total_hours;
            });
            
            // Build monthly breakdown table
            let monthlyHtml = '';
            if (monthlyData.length === 0) {
                monthlyHtml = '<p>No attendance records found for this year.</p>';
            } else {
                monthlyHtml = `
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>Month</th>
                                <th>Days Worked</th>
                                <th>Total Hours</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                `;
                
                monthlyData.forEach(monthData => {
                    monthlyHtml += `
                        <tr>
                            <td>${monthData.monthName}</td>
                            <td>${monthData.data.total_days}</td>
                            <td>${monthData.data.total_hours.toFixed(2)}</td>
                            <td>
                                <button class="btn btn-small btn-secondary month-detail-btn" 
                                        data-user-id="${userId}"
                                        data-user-name="${userName}"
                                        data-month="${monthData.month}"
                                        data-year="${year}">
                                    View Details
                                </button>
                            </td>
                        </tr>
                    `;
                });
                
                monthlyHtml += '</tbody></table>';
            }
            
            // Create modal if it doesn't exist for department page
            if (!document.getElementById('modal')) {
                const modalHtml = `
                    <div id="modal" class="modal">
                        <div class="modal-content">
                            <div id="modal-body"></div>
                        </div>
                    </div>
                `;
                document.body.insertAdjacentHTML('beforeend', modalHtml);
            }
            
            const modalContent = `
                <div class="modal-header">
                    <h3>Annual Report - ${userName}</h3>
                    <span class="close">&times;</span>
                </div>
                <div class="modal-body">
                    <div class="detail-summary">
                        <h4>${year} Annual Summary</h4>
                        <div class="summary-stats">
                            <div class="stat-item">
                                <span class="stat-label">Total Days:</span>
                                <span class="stat-value">${totalYearDays}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Total Hours:</span>
                                <span class="stat-value">${totalYearHours.toFixed(2)}</span>
                            </div>
                        </div>
                    </div>
                    <div class="detail-records">
                        <h4>Monthly Breakdown</h4>
                        ${monthlyHtml}
                    </div>
                </div>
            `;
            
            document.getElementById('modal-body').innerHTML = modalContent;
            document.getElementById('modal').style.display = 'block';
            
            // Add close functionality
            const closeBtn = document.querySelector('.modal .close');
            if (closeBtn) {
                closeBtn.onclick = () => {
                    document.getElementById('modal').style.display = 'none';
                };
            }
            
            // Add month detail button functionality
            document.querySelectorAll('.month-detail-btn').forEach(btn => {
                btn.addEventListener('click', async () => {
                    const monthUserId = btn.getAttribute('data-user-id');
                    const monthUserName = btn.getAttribute('data-user-name');
                    const month = parseInt(btn.getAttribute('data-month'));
                    const year = parseInt(btn.getAttribute('data-year'));
                    
                    await this.showUserDetail(monthUserId, monthUserName, month, year);
                });
            });
            
            // Close when clicking outside
            window.onclick = (event) => {
                const modal = document.getElementById('modal');
                if (event.target === modal) {
                    modal.style.display = 'none';
                }
            };
            
        } catch (error) {
            alert('Error loading yearly details: ' + error.message);
        }
    }
};