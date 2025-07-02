-- TracSeq 2.0 Seed Data - Part 1: Users and Authentication
-- Note: Default password for all users is 'password123'

-- Insert Permissions
INSERT INTO permissions (id, name, description) VALUES
('users.create', 'Create Users', 'Ability to create new users'),
('users.read', 'View Users', 'Ability to view user information'),
('users.update', 'Update Users', 'Ability to update user information'),
('users.delete', 'Delete Users', 'Ability to delete users'),
('samples.create', 'Create Samples', 'Ability to create new samples'),
('samples.read', 'View Samples', 'Ability to view sample information'),
('samples.update', 'Update Samples', 'Ability to update sample information'),
('samples.delete', 'Delete Samples', 'Ability to delete samples'),
('storage.manage', 'Manage Storage', 'Ability to manage storage locations'),
('sequencing.create', 'Create Sequencing Jobs', 'Ability to create sequencing jobs'),
('sequencing.manage', 'Manage Sequencing', 'Ability to manage sequencing workflows'),
('reports.view', 'View Reports', 'Ability to view reports'),
('reports.create', 'Create Reports', 'Ability to create reports'),
('admin.all', 'Administrator Access', 'Full system access')
ON CONFLICT (id) DO NOTHING;

-- Insert Roles
INSERT INTO roles (id, name, description) VALUES
('lab_technician', 'Lab Technician', 'Basic laboratory operations'),
('lab_supervisor', 'Lab Supervisor', 'Supervise lab operations and approve workflows'),
('lab_administrator', 'Lab Administrator', 'Full administrative access'),
('researcher', 'Researcher', 'Submit samples and view results'),
('quality_analyst', 'Quality Analyst', 'Perform quality control and analysis')
ON CONFLICT (id) DO NOTHING;

-- Insert Role Permissions
INSERT INTO role_permissions (role_id, permission_id) VALUES
-- Lab Technician
('lab_technician', 'samples.read'),
('lab_technician', 'samples.create'),
('lab_technician', 'samples.update'),
('lab_technician', 'storage.manage'),
-- Lab Supervisor
('lab_supervisor', 'samples.read'),
('lab_supervisor', 'samples.create'),
('lab_supervisor', 'samples.update'),
('lab_supervisor', 'samples.delete'),
('lab_supervisor', 'storage.manage'),
('lab_supervisor', 'sequencing.create'),
('lab_supervisor', 'sequencing.manage'),
('lab_supervisor', 'reports.view'),
-- Lab Administrator
('lab_administrator', 'admin.all'),
-- Researcher
('researcher', 'samples.read'),
('researcher', 'samples.create'),
('researcher', 'reports.view'),
-- Quality Analyst
('quality_analyst', 'samples.read'),
('quality_analyst', 'samples.update'),
('quality_analyst', 'reports.view'),
('quality_analyst', 'reports.create')
ON CONFLICT DO NOTHING;

-- Insert Users
INSERT INTO users (id, username, email, password_hash, full_name, is_active, created_at, updated_at) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'admin', 'admin@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'System Administrator', true, NOW() - INTERVAL '1 year', NOW()),
('550e8400-e29b-41d4-a716-446655440002', 'jsmith', 'john.smith@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'John Smith', true, NOW() - INTERVAL '6 months', NOW()),
('550e8400-e29b-41d4-a716-446655440003', 'mjohnson', 'mary.johnson@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Mary Johnson', true, NOW() - INTERVAL '8 months', NOW()),
('550e8400-e29b-41d4-a716-446655440004', 'dwilliams', 'david.williams@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'David Williams', true, NOW() - INTERVAL '4 months', NOW()),
('550e8400-e29b-41d4-a716-446655440005', 'sbrown', 'sarah.brown@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Sarah Brown', true, NOW() - INTERVAL '3 months', NOW()),
('550e8400-e29b-41d4-a716-446655440006', 'rjones', 'robert.jones@university.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Dr. Robert Jones', true, NOW() - INTERVAL '2 months', NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert User Roles
INSERT INTO user_roles (user_id, role_id) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'lab_administrator'),
('550e8400-e29b-41d4-a716-446655440002', 'lab_supervisor'),
('550e8400-e29b-41d4-a716-446655440003', 'lab_technician'),
('550e8400-e29b-41d4-a716-446655440004', 'quality_analyst'),
('550e8400-e29b-41d4-a716-446655440005', 'lab_technician'),
('550e8400-e29b-41d4-a716-446655440006', 'researcher')
ON CONFLICT DO NOTHING;
