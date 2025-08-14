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
            case 'export':
                this.setupExport();
                break;
        }
    },
    
    async loadStats() {
        const content = document.getElementById('stats-content');
        content.innerHTML = '<div class="loading">Loading statistics</div>';
        
        try {
            const response = await api.getDepartmentStats();
            if (response.success) {
                this.displayStats(response.data);
            } else {
                content.innerHTML = '<p class="error">Failed to load statistics</p>';
            }
        } catch (error) {
            content.innerHTML = '<p class="error">Error loading statistics</p>';
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
        
        content.innerHTML = html;
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
                        <th>Department</th>
                        <th>Department Name</th>
                        <th>Department Code</th>
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
                    <td>${user.department}</td>
                    <td>${user.department_name || 'N/A'}</td>
                    <td>${user.department_code || 'N/A'}</td>
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
                        <label for="department">Department:</label>
                        <input type="number" id="department" name="department" value="${user ? user.department : ''}" required>
                    </div>
                    
                    <div class="form-group">
                        <label for="department_name">Department Name:</label>
                        <input type="text" id="department_name" name="department_name" value="${user ? user.department_name || '' : ''}">
                    </div>
                    
                    <div class="form-group">
                        <label for="department_code">Department Code:</label>
                        <input type="text" id="department_code" name="department_code" value="${user ? user.department_code || '' : ''}">
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
            department: parseInt(formData.get('department')),
            department_name: formData.get('department_name') || null,
            department_code: formData.get('department_code') || null,
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
    }
};