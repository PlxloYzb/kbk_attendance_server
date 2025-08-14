// Authentication utilities
const auth = {
    // Check if user is authenticated as admin
    checkAdminAuth() {
        const token = localStorage.getItem('admin_token');
        const user = localStorage.getItem('admin_user');
        
        if (!token || !user) {
            window.location.href = '/ui/login.html';
            return false;
        }
        
        return true;
    },
    
    // Get current admin user
    getCurrentUser() {
        const userStr = localStorage.getItem('admin_user');
        return userStr ? JSON.parse(userStr) : null;
    },
    
    // Check if current user is admin
    isAdmin() {
        const user = this.getCurrentUser();
        return user && user.role === 'admin';
    },
    
    // Logout
    logout() {
        localStorage.removeItem('admin_token');
        localStorage.removeItem('admin_user');
        window.location.href = '/ui/login.html';
    },
    
    // Get auth token
    getToken() {
        return localStorage.getItem('admin_token');
    }
};