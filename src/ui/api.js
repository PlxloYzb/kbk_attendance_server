// API client for admin panel
const api = {
    baseUrl: '/admin',
    
    // Helper function to make authenticated requests
    async request(url, options = {}) {
        const token = localStorage.getItem('admin_token');
        const defaultOptions = {
            headers: {
                'Content-Type': 'application/json',
                'Authorization': token ? `Bearer ${token}` : ''
            }
        };
        
        const finalOptions = {
            ...defaultOptions,
            ...options,
            headers: {
                ...defaultOptions.headers,
                ...options.headers
            }
        };
        
        const response = await fetch(url, finalOptions);
        
        if (response.status === 401) {
            // Unauthorized - clear token and redirect to login
            localStorage.removeItem('admin_token');
            localStorage.removeItem('admin_user');
            window.location.href = '/ui/login.html';
            return;
        }
        
        return response.json();
    },
    
    // Authentication
    async adminLogin(username, password) {
        const response = await fetch(`${this.baseUrl}/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ username, password })
        });
        return response.json();
    },
    
    async getAdminInfo() {
        return this.request(`${this.baseUrl}/me`);
    },
    
    // Checkin Points
    async getCheckinPoints() {
        return this.request(`${this.baseUrl}/points/checkin`);
    },
    
    async createCheckinPoint(data) {
        return this.request(`${this.baseUrl}/points/checkin`, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },
    
    async updateCheckinPoint(id, data) {
        return this.request(`${this.baseUrl}/points/checkin/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },
    
    async deleteCheckinPoint(id) {
        return this.request(`${this.baseUrl}/points/checkin/${id}`, {
            method: 'DELETE'
        });
    },
    
    // Checkout Points
    async getCheckoutPoints() {
        return this.request(`${this.baseUrl}/points/checkout`);
    },
    
    async createCheckoutPoint(data) {
        return this.request(`${this.baseUrl}/points/checkout`, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },
    
    async updateCheckoutPoint(id, data) {
        return this.request(`${this.baseUrl}/points/checkout/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },
    
    async deleteCheckoutPoint(id) {
        return this.request(`${this.baseUrl}/points/checkout/${id}`, {
            method: 'DELETE'
        });
    },
    
    // Users
    async getUsers() {
        return this.request(`${this.baseUrl}/users`);
    },
    
    async createUser(data) {
        return this.request(`${this.baseUrl}/users`, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },
    
    async updateUser(id, data) {
        return this.request(`${this.baseUrl}/users/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },
    
    async deleteUser(id) {
        return this.request(`${this.baseUrl}/users/${id}`, {
            method: 'DELETE'
        });
    },
    
    // Checkins
    async getCheckins(filters = {}) {
        const params = new URLSearchParams();
        if (filters.user_id) params.append('user_id', filters.user_id);
        if (filters.action) params.append('action', filters.action);
        if (filters.limit) params.append('limit', filters.limit);
        
        const url = `${this.baseUrl}/checkins${params.toString() ? '?' + params.toString() : ''}`;
        return this.request(url);
    },
    
    async createCheckin(data) {
        return this.request(`${this.baseUrl}/checkins`, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },
    
    async updateCheckin(id, data) {
        return this.request(`${this.baseUrl}/checkins/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },
    
    async deleteCheckin(id) {
        return this.request(`${this.baseUrl}/checkins/${id}`, {
            method: 'DELETE'
        });
    },
    
    // Statistics
    async getDepartmentStats() {
        return this.request(`${this.baseUrl}/stats/department`);
    },

    async getFilteredDepartmentStats(params = {}) {
        const queryParams = new URLSearchParams();
        if (params.month) queryParams.append('month', params.month);
        if (params.year) queryParams.append('year', params.year);
        if (params.user_name) queryParams.append('user_name', params.user_name);
        if (params.department) queryParams.append('department', params.department);
        
        const url = `${this.baseUrl}/stats/department/filtered${queryParams.toString() ? '?' + queryParams.toString() : ''}`;
        return this.request(url);
    },

    async getUserDetail(user_id, month, year) {
        const queryParams = new URLSearchParams({
            user_id,
            month: month.toString(),
            year: year.toString()
        });
        return this.request(`${this.baseUrl}/stats/user-detail?${queryParams.toString()}`);
    },
    
    // Admin Users
    async getAdminUsers() {
        return this.request(`${this.baseUrl}/admin-users`);
    },
    
    async createAdminUser(data) {
        return this.request(`${this.baseUrl}/admin-users`, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },
    
    async updateAdminUser(id, data) {
        return this.request(`${this.baseUrl}/admin-users/${id}`, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },
    
    async deleteAdminUser(id) {
        return this.request(`${this.baseUrl}/admin-users/${id}`, {
            method: 'DELETE'
        });
    },
    
    async resetAdminPassword(id, data) {
        return this.request(`${this.baseUrl}/admin-users/${id}/password`, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },
    
    // Export
    async exportCsv() {
        const token = localStorage.getItem('admin_token');
        const response = await fetch(`${this.baseUrl}/stats/export`, {
            headers: {
                'Authorization': token ? `Bearer ${token}` : ''
            }
        });
        return response;
    }
};