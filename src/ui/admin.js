// Admin dashboard functionality
const admin = {
    currentSection: 'stats',
    
    init() {
        this.setupNavigation();
        this.setupModal();
        this.setupLogout();
        this.loadSection('stats');
    },
    
    setupNavigation() {
        const navButtons = document.querySelectorAll('.nav-btn');
        navButtons.forEach(btn => {
            btn.addEventListener('click', () => {
                const section = btn.dataset.section;
                this.switchSection(section);
            });
        });
    },
    
    setupModal() {
        const modal = document.getElementById('modal');
        const closeBtn = modal.querySelector('.close');
        
        closeBtn.addEventListener('click', () => {
            modal.style.display = 'none';
        });
        
        window.addEventListener('click', (e) => {
            if (e.target === modal) {
                modal.style.display = 'none';
            }
        });
    },
    
    setupLogout() {
        document.getElementById('logout-btn').addEventListener('click', () => {
            auth.logout();
        });
    },
    
    switchSection(section) {
        // Update navigation
        document.querySelectorAll('.nav-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelector(`[data-section="${section}"]`).classList.add('active');
        
        // Update content
        document.querySelectorAll('.content-section').forEach(sec => {
            sec.classList.remove('active');
        });
        document.getElementById(`${section}-section`).classList.add('active');
        
        this.currentSection = section;
        this.loadSection(section);
    },
    
    async loadSection(section) {
        switch (section) {
            case 'stats':
                await this.loadStats();
                break;
            case 'checkin-points':
                await this.loadCheckinPoints();
                break;
            case 'checkout-points':
                await this.loadCheckoutPoints();
                break;
            case 'users':
                await this.loadUsers();
                break;
            case 'checkins':
                await this.loadCheckins();
                break;
            case 'admin-users':
                await this.loadAdminUsers();
                break;
            case 'time-settings':
                await this.loadTimeSettings();
                break;
            case 'sync-status':
                await this.loadSyncStatus();
                break;
            case 'export':
                this.setupExport();
                break;
        }
    },
    
    async loadStats() {
        const content = document.getElementById('stats-content');
        this.setupStatsFilters(content);
        await this.loadFilteredStats();
    },

    setupStatsFilters(content) {
        const now = new Date();
        const currentYear = now.getFullYear();
        const currentMonth = now.getMonth() + 1;
        
        // Get current user info to check role
        const user = JSON.parse(localStorage.getItem('admin_user') || '{}');
        const isAdmin = user.role === 'admin';
        
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
                    
                    ${isAdmin ? `
                    <div class="filter-group">
                        <label for="stats-department">Department:</label>
                        <select id="stats-department">
                            <option value="">All Departments</option>
                            <option value="1">1 - Office</option>
                            <option value="2">2 - Mining</option>
                            <option value="3">3 - CA</option>
                            <option value="4">4 - HR</option>
                            <option value="5">5 - Warehouse</option>
                            <option value="6">6 - Lab</option>
                            <option value="7">7 - Logistics</option>
                            <option value="8">8 - Training</option>
                            <option value="9">9 - Technic</option>
                            <option value="10">10 - Hydro</option>
                            <option value="99">99 - Standby</option>
                        </select>
                    </div>
                    ` : ''}
                    
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
        const deptSelect = document.getElementById('stats-department');
        if (deptSelect) deptSelect.value = '';
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
            
            const deptSelect = document.getElementById('stats-department');
            if (deptSelect && deptSelect.value) {
                params.department = parseInt(deptSelect.value);
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
    
    displayStats(data) {
        const content = document.getElementById('stats-content');
        
        if (!data.departments || data.departments.length === 0) {
            content.innerHTML = '<p>No statistics available</p>';
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
        
        content.innerHTML = html;
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
                        ${params.department ? `<p>Department: ${params.department}</p>` : ''}
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
            
        } catch (error) {
            alert('Error loading yearly details: ' + error.message);
        }
    },
    
    async loadCheckinPoints() {
        const container = document.getElementById('checkin-points-table');
        container.innerHTML = '<div class="loading">Loading checkin points</div>';
        
        document.getElementById('add-checkin-point-btn').onclick = () => {
            this.showPointForm('checkin');
        };
        
        try {
            const response = await api.getCheckinPoints();
            if (response.success) {
                this.displayPointsTable(response.data, 'checkin', container);
            } else {
                container.innerHTML = '<p class="error">Failed to load checkin points</p>';
            }
        } catch (error) {
            container.innerHTML = '<p class="error">Error loading checkin points</p>';
        }
    },
    
    async loadCheckoutPoints() {
        const container = document.getElementById('checkout-points-table');
        container.innerHTML = '<div class="loading">Loading checkout points</div>';
        
        document.getElementById('add-checkout-point-btn').onclick = () => {
            this.showPointForm('checkout');
        };
        
        try {
            const response = await api.getCheckoutPoints();
            if (response.success) {
                this.displayPointsTable(response.data, 'checkout', container);
            } else {
                container.innerHTML = '<p class="error">Failed to load checkout points</p>';
            }
        } catch (error) {
            container.innerHTML = '<p class="error">Error loading checkout points</p>';
        }
    },
    
    displayPointsTable(points, type, container) {
        if (points.length === 0) {
            container.innerHTML = '<p>No points found</p>';
            return;
        }
        
        let html = `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>Location</th>
                        <th>Latitude</th>
                        <th>Longitude</th>
                        <th>Radius</th>
                        <th>Allowed Departments</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
        `;
        
        points.forEach(point => {
            html += `
                <tr>
                    <td>${point.id}</td>
                    <td>${point.location_name}</td>
                    <td>${point.latitude}</td>
                    <td>${point.longitude}</td>
                    <td>${point.radius}m</td>
                    <td>${point.allowed_department.join(', ')}</td>
                    <td class="actions">
                        <button class="btn btn-secondary" onclick="admin.editPoint('${type}', ${point.id})">Edit</button>
                        <button class="btn btn-danger" onclick="admin.deletePoint('${type}', ${point.id})">Delete</button>
                    </td>
                </tr>
            `;
        });
        
        html += '</tbody></table>';
        container.innerHTML = html;
    },
    
    showPointForm(type, point = null) {
        const isEdit = point !== null;
        const title = isEdit ? `Edit ${type} Point` : `Add ${type} Point`;
        
        const form = `
            <div class="modal-header">
                <h3>${title}</h3>
                <span class="close">&times;</span>
            </div>
            <div class="modal-body">
                <form id="point-form">
                    <div class="form-group">
                        <label for="location_name">Location Name:</label>
                        <input type="text" id="location_name" name="location_name" value="${point ? point.location_name : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="latitude">Latitude:</label>
                        <input type="number" id="latitude" name="latitude" step="any" value="${point ? point.latitude : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="longitude">Longitude:</label>
                        <input type="number" id="longitude" name="longitude" step="any" value="${point ? point.longitude : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="radius">Radius (meters):</label>
                        <input type="number" id="radius" name="radius" value="${point ? point.radius : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="allowed_department">Allowed Departments (comma-separated):</label>
                        <input type="text" id="allowed_department" name="allowed_department" 
                               value="${point ? point.allowed_department.join(', ') : ''}" 
                               placeholder="e.g., 1, 2, 5" required>
                    </div>
                    
                    <div class="form-actions">
                        <button type="button" class="btn btn-secondary" onclick="document.getElementById('modal').style.display='none'">Cancel</button>
                        <button type="submit" class="btn btn-primary">${isEdit ? 'Update' : 'Create'}</button>
                    </div>
                </form>
            </div>
        `;
        
        document.getElementById('modal-body').innerHTML = form;
        document.getElementById('modal').style.display = 'block';
        
        document.getElementById('point-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.submitPointForm(type, isEdit ? point.id : null);
        });
    },
    
    async submitPointForm(type, id = null) {
        const form = document.getElementById('point-form');
        const formData = new FormData(form);
        
        const data = {
            location_name: formData.get('location_name'),
            latitude: parseFloat(formData.get('latitude')),
            longitude: parseFloat(formData.get('longitude')),
            radius: parseFloat(formData.get('radius')),
            allowed_department: formData.get('allowed_department').split(',').map(d => parseInt(d.trim())).filter(d => !isNaN(d))
        };
        
        try {
            let response;
            if (id) {
                if (type === 'checkin') {
                    response = await api.updateCheckinPoint(id, data);
                } else {
                    response = await api.updateCheckoutPoint(id, data);
                }
            } else {
                if (type === 'checkin') {
                    response = await api.createCheckinPoint(data);
                } else {
                    response = await api.createCheckoutPoint(data);
                }
            }
            
            if (response.success) {
                document.getElementById('modal').style.display = 'none';
                if (type === 'checkin') {
                    this.loadCheckinPoints();
                } else {
                    this.loadCheckoutPoints();
                }
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error saving point: ' + error.message);
        }
    },
    
    async editPoint(type, id) {
        try {
            let response;
            if (type === 'checkin') {
                response = await api.getCheckinPoints();
            } else {
                response = await api.getCheckoutPoints();
            }
            
            if (response.success) {
                const point = response.data.find(p => p.id === id);
                if (point) {
                    this.showPointForm(type, point);
                }
            }
        } catch (error) {
            alert('Error loading point: ' + error.message);
        }
    },
    
    async deletePoint(type, id) {
        if (!confirm('Are you sure you want to delete this point?')) {
            return;
        }
        
        try {
            let response;
            if (type === 'checkin') {
                response = await api.deleteCheckinPoint(id);
            } else {
                response = await api.deleteCheckoutPoint(id);
            }
            
            if (response.success) {
                if (type === 'checkin') {
                    this.loadCheckinPoints();
                } else {
                    this.loadCheckoutPoints();
                }
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error deleting point: ' + error.message);
        }
    },
    
    async loadUsers() {
        const container = document.getElementById('users-table');
        container.innerHTML = '<div class="loading">Loading users</div>';
        
        document.getElementById('add-user-btn').onclick = () => {
            this.showUserForm();
        };
        
        try {
            const response = await api.getUsers();
            if (response.success) {
                this.displayUsersTable(response.data, container);
            } else {
                container.innerHTML = '<p class="error">Failed to load users</p>';
            }
        } catch (error) {
            container.innerHTML = '<p class="error">Error loading users</p>';
        }
    },
    
    displayUsersTable(users, container) {
        if (users.length === 0) {
            container.innerHTML = '<p>No users found</p>';
            return;
        }
        
        let html = `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>User ID</th>
                        <th>User Name</th>
                        <th>Department</th>
                        <th>Department Name</th>
                        <th>Passkey</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
        `;
        
        users.forEach(user => {
            html += `
                <tr>
                    <td>${user.id}</td>
                    <td>${user.user_id}</td>
                    <td>${user.user_name || 'N/A'}</td>
                    <td>${user.department}</td>
                    <td>${user.department_name || 'N/A'}</td>
                    <td>${user.passkey}</td>
                    <td class="actions">
                        <button class="btn btn-secondary" onclick="admin.editUser(${user.id})">Edit</button>
                        <button class="btn btn-danger" onclick="admin.deleteUser(${user.id})">Delete</button>
                    </td>
                </tr>
            `;
        });
        
        html += '</tbody></table>';
        container.innerHTML = html;
    },
    
    showUserForm(user = null) {
        const isEdit = user !== null;
        const title = isEdit ? 'Edit User' : 'Add User';
        
        const form = `
            <div class="modal-header">
                <h3>${title}</h3>
                <span class="close">&times;</span>
            </div>
            <div class="modal-body">
                <form id="user-form">
                    <div class="form-group">
                        <label for="user_id">User ID:</label>
                        <input type="text" id="user_id" name="user_id" value="${user ? user.user_id : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="user_name">User Name (姓名):</label>
                        <input type="text" id="user_name" name="user_name" value="${user ? user.user_name || '' : ''}" placeholder="请输入中文姓名">
                    </div>
                    
                    <div class="form-group">
                        <label for="department">Department:</label>
                        <input type="number" id="department" name="department" value="${user ? user.department : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="department_name">Department Name:</label>
                        <input type="text" id="department_name" name="department_name" value="${user ? user.department_name || '' : ''}">
                    </div>
                    
                    
                    <div class="form-group">
                        <label for="passkey">Passkey:</label>
                        <input type="text" id="passkey" name="passkey" value="${user ? user.passkey : ''}" required>
                    </div>
                    
                    <div class="form-actions">
                        <button type="button" class="btn btn-secondary" onclick="document.getElementById('modal').style.display='none'">Cancel</button>
                        <button type="submit" class="btn btn-primary">${isEdit ? 'Update' : 'Create'}</button>
                    </div>
                </form>
            </div>
        `;
        
        document.getElementById('modal-body').innerHTML = form;
        document.getElementById('modal').style.display = 'block';
        
        document.getElementById('user-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.submitUserForm(isEdit ? user.id : null);
        });
    },
    
    async submitUserForm(id = null) {
        const form = document.getElementById('user-form');
        const formData = new FormData(form);
        
        const data = {
            user_id: formData.get('user_id'),
            user_name: formData.get('user_name') || null,
            department: parseInt(formData.get('department')),
            department_name: formData.get('department_name') || null,
            passkey: formData.get('passkey')
        };
        
        try {
            let response;
            if (id) {
                response = await api.updateUser(id, data);
            } else {
                response = await api.createUser(data);
            }
            
            if (response.success) {
                document.getElementById('modal').style.display = 'none';
                this.loadUsers();
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error saving user: ' + error.message);
        }
    },
    
    async editUser(id) {
        try {
            const response = await api.getUsers();
            if (response.success) {
                const user = response.data.find(u => u.id === id);
                if (user) {
                    this.showUserForm(user);
                }
            }
        } catch (error) {
            alert('Error loading user: ' + error.message);
        }
    },
    
    async deleteUser(id) {
        if (!confirm('Are you sure you want to delete this user?')) {
            return;
        }
        
        try {
            const response = await api.deleteUser(id);
            if (response.success) {
                this.loadUsers();
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error deleting user: ' + error.message);
        }
    },
    
    async loadCheckins() {
        const container = document.getElementById('checkins-table');
        container.innerHTML = '<div class="loading">Loading checkins</div>';
        
        document.getElementById('add-checkin-btn').onclick = () => {
            this.showCheckinForm();
        };
        
        document.getElementById('apply-checkin-filter').onclick = () => {
            this.applyCheckinFilter();
        };
        
        try {
            const response = await api.getCheckins();
            if (response.success) {
                this.displayCheckinsTable(response.data, container);
            } else {
                container.innerHTML = '<p class="error">Failed to load checkins</p>';
            }
        } catch (error) {
            container.innerHTML = '<p class="error">Error loading checkins</p>';
        }
    },
    
    async applyCheckinFilter() {
        const userFilter = document.getElementById('checkin-user-filter').value;
        const actionFilter = document.getElementById('checkin-action-filter').value;
        
        const filters = {};
        if (userFilter) filters.user_id = userFilter;
        if (actionFilter) filters.action = actionFilter;
        filters.limit = 100;
        
        const container = document.getElementById('checkins-table');
        container.innerHTML = '<div class="loading">Loading checkins</div>';
        
        try {
            const response = await api.getCheckins(filters);
            if (response.success) {
                this.displayCheckinsTable(response.data, container);
            } else {
                container.innerHTML = '<p class="error">Failed to load checkins</p>';
            }
        } catch (error) {
            container.innerHTML = '<p class="error">Error loading checkins</p>';
        }
    },
    
    displayCheckinsTable(checkins, container) {
        if (checkins.length === 0) {
            container.innerHTML = '<p>No checkins found</p>';
            return;
        }
        
        let html = `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>User ID</th>
                        <th>Action</th>
                        <th>Created At</th>
                        <th>Latitude</th>
                        <th>Longitude</th>
                        <th>Synced</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
        `;
        
        checkins.forEach(checkin => {
            const createdAt = new Date(checkin.created_at).toLocaleString();
            html += `
                <tr>
                    <td>${checkin.id}</td>
                    <td>${checkin.user_id}</td>
                    <td><span class="${checkin.action === 'IN' ? 'badge-in' : 'badge-out'}">${checkin.action}</span></td>
                    <td>${createdAt}</td>
                    <td>${checkin.latitude || 'N/A'}</td>
                    <td>${checkin.longitude || 'N/A'}</td>
                    <td>${checkin.is_synced ? 'Yes' : 'No'}</td>
                    <td class="actions">
                        <button class="btn btn-secondary" onclick="admin.editCheckin(${checkin.id})">Edit</button>
                        <button class="btn btn-danger" onclick="admin.deleteCheckin(${checkin.id})">Delete</button>
                    </td>
                </tr>
            `;
        });
        
        html += '</tbody></table>';
        container.innerHTML = html;
    },
    
    showCheckinForm(checkin = null) {
        const isEdit = checkin !== null;
        const title = isEdit ? 'Edit Checkin' : 'Add Checkin';
        
        const createdAt = checkin ? 
            new Date(checkin.created_at).toISOString().slice(0, 19) : 
            new Date().toISOString().slice(0, 19);
        
        const form = `
            <div class="modal-header">
                <h3>${title}</h3>
                <span class="close">&times;</span>
            </div>
            <div class="modal-body">
                <form id="checkin-form">
                    <div class="form-group">
                        <label for="user_id">User ID:</label>
                        <input type="text" id="user_id" name="user_id" value="${checkin ? checkin.user_id : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="action">Action:</label>
                        <select id="action" name="action" required>
                            <option value="IN" ${checkin && checkin.action === 'IN' ? 'selected' : ''}>Check In</option>
                            <option value="OUT" ${checkin && checkin.action === 'OUT' ? 'selected' : ''}>Check Out</option>
                        </select>
                    </div>
                    
                    <div class="form-group">
                        <label for="created_at">Created At:</label>
                        <input type="datetime-local" id="created_at" name="created_at" value="${createdAt}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="latitude">Latitude:</label>
                        <input type="number" id="latitude" name="latitude" step="any" value="${checkin ? checkin.latitude || '' : ''}">
                    </div>
                    
                    <div class="form-group">
                        <label for="longitude">Longitude:</label>
                        <input type="number" id="longitude" name="longitude" step="any" value="${checkin ? checkin.longitude || '' : ''}">
                    </div>
                    
                    <div class="form-group">
                        <label for="is_synced">Synced:</label>
                        <select id="is_synced" name="is_synced">
                            <option value="0" ${checkin && checkin.is_synced === 0 ? 'selected' : ''}>No</option>
                            <option value="1" ${checkin && checkin.is_synced === 1 ? 'selected' : ''}>Yes</option>
                        </select>
                    </div>
                    
                    <div class="form-actions">
                        <button type="button" class="btn btn-secondary" onclick="document.getElementById('modal').style.display='none'">Cancel</button>
                        <button type="submit" class="btn btn-primary">${isEdit ? 'Update' : 'Create'}</button>
                    </div>
                </form>
            </div>
        `;
        
        document.getElementById('modal-body').innerHTML = form;
        document.getElementById('modal').style.display = 'block';
        
        document.getElementById('checkin-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.submitCheckinForm(isEdit ? checkin.id : null);
        });
    },
    
    async submitCheckinForm(id = null) {
        const form = document.getElementById('checkin-form');
        const formData = new FormData(form);
        
        const data = {
            user_id: formData.get('user_id'),
            action: formData.get('action'),
            created_at: new Date(formData.get('created_at')).toISOString(),
            latitude: formData.get('latitude') ? parseFloat(formData.get('latitude')) : null,
            longitude: formData.get('longitude') ? parseFloat(formData.get('longitude')) : null,
            is_synced: parseInt(formData.get('is_synced'))
        };
        
        try {
            let response;
            if (id) {
                response = await api.updateCheckin(id, data);
            } else {
                response = await api.createCheckin(data);
            }
            
            if (response.success) {
                document.getElementById('modal').style.display = 'none';
                this.loadCheckins();
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error saving checkin: ' + error.message);
        }
    },
    
    async editCheckin(id) {
        try {
            const response = await api.getCheckins();
            if (response.success) {
                const checkin = response.data.find(c => c.id === id);
                if (checkin) {
                    this.showCheckinForm(checkin);
                }
            }
        } catch (error) {
            alert('Error loading checkin: ' + error.message);
        }
    },
    
    async deleteCheckin(id) {
        if (!confirm('Are you sure you want to delete this checkin?')) {
            return;
        }
        
        try {
            const response = await api.deleteCheckin(id);
            if (response.success) {
                this.loadCheckins();
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error deleting checkin: ' + error.message);
        }
    },
    
    setupExport() {
        document.getElementById('export-csv-btn').onclick = async () => {
            try {
                const response = await api.exportCsv();
                if (response.ok) {
                    const blob = await response.blob();
                    const url = window.URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.style.display = 'none';
                    a.href = url;
                    a.download = 'attendance_export.csv';
                    document.body.appendChild(a);
                    a.click();
                    window.URL.revokeObjectURL(url);
                } else {
                    alert('Export failed');
                }
            } catch (error) {
                alert('Export failed: ' + error.message);
            }
        };
    },
    
    // Admin Users Management
    async loadAdminUsers() {
        const container = document.getElementById('admin-users-table');
        container.innerHTML = '<div class="loading">Loading admin users...</div>';
        
        document.getElementById('add-admin-user-btn').onclick = () => {
            this.showAdminUserForm();
        };
        
        try {
            const response = await api.getAdminUsers();
            if (response.success) {
                this.displayAdminUsersTable(response.data, container);
            } else {
                container.innerHTML = '<p class="error">Failed to load admin users</p>';
            }
        } catch (error) {
            container.innerHTML = '<p class="error">Error loading admin users</p>';
        }
    },
    
    displayAdminUsersTable(adminUsers, container) {
        if (!adminUsers || adminUsers.length === 0) {
            container.innerHTML = '<p>No admin users found</p>';
            return;
        }
        
        let html = `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>Username</th>
                        <th>Role</th>
                        <th>Department</th>
                        <th>Created At</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
        `;
        
        adminUsers.forEach(user => {
            const createdAt = new Date(user.created_at).toLocaleString();
            const departmentDisplay = user.department || 'N/A';
            
            html += `
                <tr>
                    <td>${user.id}</td>
                    <td>${user.username}</td>
                    <td>${user.role}</td>
                    <td>${departmentDisplay}</td>
                    <td>${createdAt}</td>
                    <td class="actions">
                        <button class="btn btn-secondary" onclick="admin.editAdminUser(${user.id})">Edit</button>
                        <button class="btn btn-warning" onclick="admin.resetAdminPassword(${user.id})">Reset Password</button>
                        <button class="btn btn-danger" onclick="admin.deleteAdminUser(${user.id})">Delete</button>
                    </td>
                </tr>
            `;
        });
        
        html += '</tbody></table>';
        container.innerHTML = html;
    },
    
    showAdminUserForm(user = null) {
        const isEdit = user !== null;
        const title = isEdit ? 'Edit Admin User' : 'Create Admin User';
        
        const form = `
            <div class="modal-header">
                <h3>${title}</h3>
                <span class="close">&times;</span>
            </div>
            <div class="modal-body">
                <form id="admin-user-form">
                    <div class="form-group">
                        <label for="admin_username">Username:</label>
                        <input type="text" id="admin_username" name="username" value="${user ? user.username : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="admin_password">Password:</label>
                        <input type="password" id="admin_password" name="password" ${isEdit ? '' : 'required'}>
                        ${isEdit ? '<small>Leave empty to keep current password</small>' : ''}
                    </div>
                    
                    <div class="form-group">
                        <label for="admin_role">Role:</label>
                        <select id="admin_role" name="role" required>
                            <option value="admin" ${user && user.role === 'admin' ? 'selected' : ''}>Admin</option>
                            <option value="department" ${user && user.role === 'department' ? 'selected' : ''}>Department</option>
                        </select>
                    </div>
                    
                    <div class="form-group">
                        <label for="admin_department">Department:</label>
                        <select id="admin_department" name="department">
                            <option value="">None (Admin only)</option>
                            <option value="1" ${user && user.department === 1 ? 'selected' : ''}>1 - Office</option>
                            <option value="2" ${user && user.department === 2 ? 'selected' : ''}>2 - Mining</option>
                            <option value="3" ${user && user.department === 3 ? 'selected' : ''}>3 - CA</option>
                            <option value="4" ${user && user.department === 4 ? 'selected' : ''}>4 - HR</option>
                            <option value="5" ${user && user.department === 5 ? 'selected' : ''}>5 - Warehouse</option>
                            <option value="6" ${user && user.department === 6 ? 'selected' : ''}>6 - Lab</option>
                            <option value="7" ${user && user.department === 7 ? 'selected' : ''}>7 - Logistics</option>
                            <option value="8" ${user && user.department === 8 ? 'selected' : ''}>8 - Training</option>
                            <option value="9" ${user && user.department === 9 ? 'selected' : ''}>9 - Technic</option>
                            <option value="10" ${user && user.department === 10 ? 'selected' : ''}>10 - Hydro</option>
                            <option value="99" ${user && user.department === 99 ? 'selected' : ''}>99 - Standby</option>
                        </select>
                    </div>
                    
                    <div class="form-actions">
                        <button type="button" class="btn btn-secondary" onclick="document.getElementById('modal').style.display='none'">Cancel</button>
                        <button type="submit" class="btn btn-primary">${isEdit ? 'Update' : 'Create'}</button>
                    </div>
                </form>
            </div>
        `;
        
        document.getElementById('modal-body').innerHTML = form;
        document.getElementById('modal').style.display = 'block';
        
        // Handle role change to show/hide department field
        document.getElementById('admin_role').addEventListener('change', function() {
            const deptField = document.getElementById('admin_department');
            const deptGroup = deptField.closest('.form-group');
            if (this.value === 'department') {
                deptGroup.style.display = 'block';
                deptField.required = true;
            } else {
                deptGroup.style.display = 'none';
                deptField.required = false;
                deptField.value = '';
            }
        });
        
        // Trigger role change event to set initial state
        document.getElementById('admin_role').dispatchEvent(new Event('change'));
        
        // Handle form submission
        document.getElementById('admin-user-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            
            const formData = new FormData(e.target);
            const data = {
                username: formData.get('username'),
                role: formData.get('role'),
                department: formData.get('department') ? parseInt(formData.get('department')) : null
            };
            
            if (formData.get('password')) {
                data.password = formData.get('password');
            }
            
            await this.saveAdminUser(user ? user.id : null, data);
        });
        
        // Handle close button
        document.querySelector('.close').onclick = () => {
            document.getElementById('modal').style.display = 'none';
        };
    },
    
    async saveAdminUser(id, data) {
        try {
            let response;
            if (id) {
                response = await api.updateAdminUser(id, data);
            } else {
                response = await api.createAdminUser(data);
            }
            
            if (response.success) {
                document.getElementById('modal').style.display = 'none';
                this.loadAdminUsers();
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error saving admin user: ' + error.message);
        }
    },
    
    async editAdminUser(id) {
        try {
            const response = await api.getAdminUsers();
            if (response.success) {
                const user = response.data.find(u => u.id === id);
                if (user) {
                    this.showAdminUserForm(user);
                } else {
                    alert('Admin user not found');
                }
            } else {
                alert('Error loading admin user: ' + response.message);
            }
        } catch (error) {
            alert('Error loading admin user: ' + error.message);
        }
    },
    
    async deleteAdminUser(id) {
        if (!confirm('Are you sure you want to delete this admin user?')) {
            return;
        }
        
        try {
            const response = await api.deleteAdminUser(id);
            if (response.success) {
                this.loadAdminUsers();
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error deleting admin user: ' + error.message);
        }
    },
    
    async resetAdminPassword(id) {
        const newPassword = prompt('Enter new password:');
        if (!newPassword) {
            return;
        }
        
        if (newPassword.length < 6) {
            alert('Password must be at least 6 characters long');
            return;
        }
        
        try {
            const response = await api.resetAdminPassword(id, { new_password: newPassword });
            if (response.success) {
                alert('Password reset successfully');
            } else {
                alert('Error: ' + response.message);
            }
        } catch (error) {
            alert('Error resetting password: ' + error.message);
        }
    },

    // Time Settings Management
    async loadTimeSettings() {
        const loadBtn = document.getElementById('load-time-settings');
        const saveBtn = document.getElementById('save-time-settings');
        const deptSelect = document.getElementById('time-settings-department');
        
        loadBtn.onclick = () => this.loadUsersWithTimeSettings();
        saveBtn.onclick = () => this.saveTimeSettings();
        
        // Auto-load all users initially
        await this.loadUsersWithTimeSettings();
    },

    async loadUsersWithTimeSettings() {
        const content = document.getElementById('time-settings-content');
        const deptSelect = document.getElementById('time-settings-department');
        const saveBtn = document.getElementById('save-time-settings');
        
        content.innerHTML = '<div class="loading">Loading users...</div>';
        saveBtn.disabled = true;
        
        try {
            const params = {};
            if (deptSelect.value) {
                params.department = deptSelect.value;
            }
            
            const response = await api.getUsersWithTimeSettings(params);
            if (response.success) {
                this.displayUsersWithTimeSettings(response.data);
                saveBtn.disabled = false;
            } else {
                content.innerHTML = '<p class="error">Failed to load users</p>';
            }
        } catch (error) {
            content.innerHTML = '<p class="error">Error loading users</p>';
        }
    },

    displayUsersWithTimeSettings(users) {
        const content = document.getElementById('time-settings-content');
        
        if (users.length === 0) {
            content.innerHTML = '<p>No users found for the selected department</p>';
            return;
        }

        let html = `
            <div class="time-settings-table-container">
                <table class="data-table time-settings-table">
                    <thead>
                        <tr>
                            <th>
                                <input type="checkbox" id="select-all-users" title="Select All">
                            </th>
                            <th>User Name</th>
                            <th>User ID</th>
                            <th>Department</th>
                            <th>On Duty Time</th>
                            <th>Off Duty Time</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
        `;

        users.forEach(user => {
            const onDutyTime = user.on_duty_time || '07:30:00';
            const offDutyTime = user.off_duty_time || '17:00:00';
            const hasCustomSettings = user.on_duty_time !== null;
            
            html += `
                <tr data-user-id="${user.user_id}">
                    <td>
                        <input type="checkbox" class="user-checkbox" value="${user.user_id}">
                    </td>
                    <td>${user.user_name || user.user_id}</td>
                    <td>${user.user_id}</td>
                    <td>${user.department} - ${user.department_name || 'N/A'}</td>
                    <td>
                        <input type="time" class="time-input on-duty-time" 
                               value="${onDutyTime.substring(0, 5)}" 
                               data-user-id="${user.user_id}">
                    </td>
                    <td>
                        <input type="time" class="time-input off-duty-time" 
                               value="${offDutyTime.substring(0, 5)}" 
                               data-user-id="${user.user_id}">
                    </td>
                    <td>
                        <span class="status-badge ${hasCustomSettings ? 'custom' : 'default'}">
                            ${hasCustomSettings ? 'Custom' : 'Default'}
                        </span>
                    </td>
                    <td>
                        <button class="btn btn-small btn-secondary apply-batch-btn" 
                                data-user-id="${user.user_id}">
                            Apply to Selected
                        </button>
                        ${hasCustomSettings ? `
                        <button class="btn btn-small btn-danger reset-btn" 
                                data-user-id="${user.user_id}">
                            Reset
                        </button>
                        ` : ''}
                    </td>
                </tr>
            `;
        });

        html += `
                    </tbody>
                </table>
            </div>
            
            <div class="time-settings-actions">
                <div class="batch-actions">
                    <h4>Batch Actions:</h4>
                    <div class="batch-controls">
                        <div class="time-group">
                            <label>On Duty:</label>
                            <input type="time" id="batch-on-duty" value="07:30">
                        </div>
                        <div class="time-group">
                            <label>Off Duty:</label>
                            <input type="time" id="batch-off-duty" value="17:00">
                        </div>
                        <button id="apply-batch-times" class="btn btn-secondary">
                            Apply to Selected Users
                        </button>
                    </div>
                </div>
            </div>
        `;

        content.innerHTML = html;
        
        // Add event listeners
        this.setupTimeSettingsEvents();
    },

    setupTimeSettingsEvents() {
        // Select all checkbox
        const selectAllCheckbox = document.getElementById('select-all-users');
        if (selectAllCheckbox) {
            selectAllCheckbox.addEventListener('change', () => {
                const userCheckboxes = document.querySelectorAll('.user-checkbox');
                userCheckboxes.forEach(cb => cb.checked = selectAllCheckbox.checked);
            });
        }

        // Individual apply to selected buttons
        document.querySelectorAll('.apply-batch-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const userId = btn.getAttribute('data-user-id');
                this.applyTimeToSelected(userId);
            });
        });

        // Reset buttons
        document.querySelectorAll('.reset-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const userId = btn.getAttribute('data-user-id');
                this.resetUserTimeSetting(userId);
            });
        });

        // Batch apply button
        const batchApplyBtn = document.getElementById('apply-batch-times');
        if (batchApplyBtn) {
            batchApplyBtn.addEventListener('click', () => {
                this.applyBatchTimes();
            });
        }

        // Time input change tracking
        document.querySelectorAll('.time-input').forEach(input => {
            input.addEventListener('change', () => {
                this.markAsChanged();
            });
        });
    },

    applyTimeToSelected(sourceUserId) {
        const sourceRow = document.querySelector(`tr[data-user-id="${sourceUserId}"]`);
        const onDutyTime = sourceRow.querySelector('.on-duty-time').value;
        const offDutyTime = sourceRow.querySelector('.off-duty-time').value;
        
        const selectedCheckboxes = document.querySelectorAll('.user-checkbox:checked');
        
        if (selectedCheckboxes.length === 0) {
            alert('Please select users to apply the time settings to.');
            return;
        }

        selectedCheckboxes.forEach(checkbox => {
            const userId = checkbox.value;
            const targetRow = document.querySelector(`tr[data-user-id="${userId}"]`);
            if (targetRow) {
                targetRow.querySelector('.on-duty-time').value = onDutyTime;
                targetRow.querySelector('.off-duty-time').value = offDutyTime;
            }
        });

        this.markAsChanged();
        alert(`Applied time settings to ${selectedCheckboxes.length} users.`);
    },

    applyBatchTimes() {
        const batchOnDuty = document.getElementById('batch-on-duty').value;
        const batchOffDuty = document.getElementById('batch-off-duty').value;
        const selectedCheckboxes = document.querySelectorAll('.user-checkbox:checked');
        
        if (selectedCheckboxes.length === 0) {
            alert('Please select users to apply the batch time settings to.');
            return;
        }

        selectedCheckboxes.forEach(checkbox => {
            const userId = checkbox.value;
            const targetRow = document.querySelector(`tr[data-user-id="${userId}"]`);
            if (targetRow) {
                targetRow.querySelector('.on-duty-time').value = batchOnDuty;
                targetRow.querySelector('.off-duty-time').value = batchOffDuty;
            }
        });

        this.markAsChanged();
        alert(`Applied batch time settings to ${selectedCheckboxes.length} users.`);
    },

    async resetUserTimeSetting(userId) {
        if (!confirm(`Reset time settings for user ${userId} to default (7:30-17:00)?`)) {
            return;
        }

        try {
            const response = await api.deleteTimeSetting(userId);
            if (response.success) {
                // Update UI to show default times
                const row = document.querySelector(`tr[data-user-id="${userId}"]`);
                if (row) {
                    row.querySelector('.on-duty-time').value = '07:30';
                    row.querySelector('.off-duty-time').value = '17:00';
                    const statusBadge = row.querySelector('.status-badge');
                    statusBadge.textContent = 'Default';
                    statusBadge.className = 'status-badge default';
                    
                    // Remove reset button
                    const resetBtn = row.querySelector('.reset-btn');
                    if (resetBtn) resetBtn.remove();
                }
                alert('Time setting reset to default successfully.');
            } else {
                alert('Failed to reset time setting: ' + response.message);
            }
        } catch (error) {
            alert('Error resetting time setting: ' + error.message);
        }
    },

    markAsChanged() {
        const saveBtn = document.getElementById('save-time-settings');
        if (saveBtn) {
            saveBtn.disabled = false;
            saveBtn.textContent = 'Save Changes *';
            saveBtn.classList.add('btn-warning');
        }
    },

    async saveTimeSettings() {
        const saveBtn = document.getElementById('save-time-settings');
        const originalText = saveBtn.textContent;
        
        saveBtn.disabled = true;
        saveBtn.textContent = 'Saving...';
        
        try {
            const settings = [];
            const timeInputs = document.querySelectorAll('.time-input');
            const processedUsers = new Set();
            
            timeInputs.forEach(input => {
                const userId = input.getAttribute('data-user-id');
                if (processedUsers.has(userId)) return;
                
                const row = document.querySelector(`tr[data-user-id="${userId}"]`);
                const onDutyTime = row.querySelector('.on-duty-time').value + ':00';
                const offDutyTime = row.querySelector('.off-duty-time').value + ':00';
                
                settings.push({
                    user_id: userId,
                    on_duty_time: onDutyTime,
                    off_duty_time: offDutyTime
                });
                
                processedUsers.add(userId);
            });
            
            const response = await api.batchUpdateTimeSettings(settings);
            if (response.success) {
                alert(`Successfully saved time settings for ${settings.length} users.`);
                saveBtn.textContent = 'Save Changes';
                saveBtn.classList.remove('btn-warning');
                
                // Reload to refresh status badges
                await this.loadUsersWithTimeSettings();
            } else {
                alert('Failed to save time settings: ' + response.message);
            }
        } catch (error) {
            alert('Error saving time settings: ' + error.message);
        } finally {
            saveBtn.disabled = false;
            if (saveBtn.textContent === 'Saving...') {
                saveBtn.textContent = originalText;
            }
        }
    },

    // Sync Status Management
    async loadSyncStatus() {
        await this.updateSyncStatus();
        this.setupSyncEvents();
    },

    async updateSyncStatus() {
        try {
            const response = await api.getSyncStatus();
            if (response.success) {
                const status = response.data;
                this.displaySyncStatus(status);
            } else {
                console.error('Failed to get sync status:', response.message);
                this.displaySyncError('Failed to load sync status');
            }
        } catch (error) {
            console.error('Error getting sync status:', error);
            this.displaySyncError('Error loading sync status');
        }
    },

    displaySyncStatus(status) {
        document.getElementById('total-users-count').textContent = status.total_users;
        document.getElementById('users-with-settings-count').textContent = status.users_with_time_settings;
        
        const indicator = document.getElementById('sync-status-indicator');
        const missingInfo = document.getElementById('missing-users-info');
        const missingCount = document.getElementById('missing-users-count');
        
        if (status.is_synced) {
            indicator.textContent = 'Synced';
            indicator.className = 'status-badge synced';
            missingInfo.style.display = 'none';
        } else {
            indicator.textContent = 'Out of Sync';
            indicator.className = 'status-badge out-of-sync';
            missingInfo.style.display = 'block';
            missingCount.textContent = status.missing_count;
        }
    },

    displaySyncError(message) {
        document.getElementById('total-users-count').textContent = 'Error';
        document.getElementById('users-with-settings-count').textContent = 'Error';
        document.getElementById('sync-status-indicator').textContent = message;
        document.getElementById('sync-status-indicator').className = 'status-badge error';
    },

    setupSyncEvents() {
        // Check status button
        document.getElementById('check-sync-status-btn').addEventListener('click', () => {
            this.updateSyncStatus();
        });
        
        // Manual sync button
        document.getElementById('manual-sync-btn').addEventListener('click', () => {
            this.performManualSync();
        });
    },

    async performManualSync() {
        const syncBtn = document.getElementById('manual-sync-btn');
        const resultDiv = document.getElementById('sync-result');
        
        try {
            syncBtn.disabled = true;
            syncBtn.textContent = '🔄 Syncing...';
            resultDiv.style.display = 'none';
            
            const response = await api.manualSyncTimeSettings();
            
            if (response.success) {
                const result = response.data;
                this.displaySyncResult({
                    success: true,
                    message: result.message,
                    count: result.synced_count,
                    timestamp: result.timestamp
                });
                
                // Refresh status after sync
                await this.updateSyncStatus();
            } else {
                this.displaySyncResult({
                    success: false,
                    message: response.message || 'Sync failed'
                });
            }
        } catch (error) {
            this.displaySyncResult({
                success: false,
                message: 'Error during sync: ' + error.message
            });
        } finally {
            syncBtn.disabled = false;
            syncBtn.textContent = '🔄 Manual Sync';
        }
    },

    displaySyncResult(result) {
        const resultDiv = document.getElementById('sync-result');
        
        if (result.success) {
            resultDiv.innerHTML = `
                <div class="sync-success">
                    <h4>✅ Sync Completed Successfully</h4>
                    <p>${result.message}</p>
                    ${result.count > 0 ? `<p><strong>${result.count}</strong> users were synced with default time settings.</p>` : ''}
                    ${result.timestamp ? `<p><small>Completed at: ${new Date(result.timestamp).toLocaleString()}</small></p>` : ''}
                </div>
            `;
        } else {
            resultDiv.innerHTML = `
                <div class="sync-error">
                    <h4>❌ Sync Failed</h4>
                    <p>${result.message}</p>
                </div>
            `;
        }
        
        resultDiv.style.display = 'block';
        
        // Auto-hide after 10 seconds
        setTimeout(() => {
            if (resultDiv.style.display !== 'none') {
                resultDiv.style.display = 'none';
            }
        }, 10000);
    }
};