-- Notification Service - Initial Schema Migration
-- File: notification_service/migrations/001_initial_notification_schema.sql

-- Create custom types
CREATE TYPE notification_channel AS ENUM ('email', 'slack', 'sms', 'webhook', 'push', 'in_app');
CREATE TYPE notification_priority AS ENUM ('low', 'normal', 'high', 'urgent', 'critical');
CREATE TYPE notification_status AS ENUM ('pending', 'sent', 'delivered', 'failed', 'retrying', 'cancelled');
CREATE TYPE template_type AS ENUM ('alert', 'reminder', 'report', 'status_update', 'maintenance', 'compliance');

-- Notification Templates table
CREATE TABLE notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    template_type template_type NOT NULL,
    subject_template TEXT NOT NULL,
    body_template TEXT NOT NULL,
    html_template TEXT,
    supported_channels notification_channel[] NOT NULL,
    default_priority notification_priority DEFAULT 'normal',
    variables JSONB DEFAULT '[]', -- Array of variable names used in template
    template_metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    version INTEGER DEFAULT 1,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT notification_templates_version_check CHECK (version > 0)
);

-- Notification Recipients table (groups and individual contacts)
CREATE TABLE notification_recipients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL, -- 'user', 'group', 'role', 'external'
    email VARCHAR(255),
    phone VARCHAR(50),
    slack_user_id VARCHAR(100),
    slack_channel VARCHAR(100),
    webhook_url TEXT,
    push_token TEXT,
    preferred_channels notification_channel[] DEFAULT ARRAY['email'],
    is_active BOOLEAN DEFAULT true,
    timezone VARCHAR(50) DEFAULT 'UTC',
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    recipient_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Notification Rules table (routing and escalation rules)
CREATE TABLE notification_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rule_type VARCHAR(100) NOT NULL, -- 'alert_routing', 'escalation', 'schedule', 'compliance'
    conditions JSONB NOT NULL, -- Rule conditions (e.g., {"severity": "high", "service": "storage"})
    actions JSONB NOT NULL, -- Actions to take (channels, recipients, delays)
    priority notification_priority DEFAULT 'normal',
    is_active BOOLEAN DEFAULT true,
    execution_order INTEGER DEFAULT 100,
    last_triggered TIMESTAMPTZ,
    trigger_count INTEGER DEFAULT 0,
    rule_metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT notification_rules_order_check CHECK (execution_order > 0)
);

-- Notifications table (individual notification instances)
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID,
    rule_id UUID,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    html_body TEXT,
    priority notification_priority DEFAULT 'normal',
    channel notification_channel NOT NULL,
    recipient_id UUID,
    recipient_address TEXT NOT NULL, -- email, phone, slack ID, etc.
    status notification_status DEFAULT 'pending',
    scheduled_for TIMESTAMPTZ DEFAULT NOW(),
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    context_data JSONB DEFAULT '{}', -- Data used to render template
    correlation_id VARCHAR(255), -- For grouping related notifications
    source_service VARCHAR(100),
    source_event_id UUID,
    external_id VARCHAR(255), -- ID from external service (email service, Slack, etc.)
    delivery_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT notifications_template_fkey FOREIGN KEY (template_id) REFERENCES notification_templates(id),
    CONSTRAINT notifications_rule_fkey FOREIGN KEY (rule_id) REFERENCES notification_rules(id),
    CONSTRAINT notifications_recipient_fkey FOREIGN KEY (recipient_id) REFERENCES notification_recipients(id),
    CONSTRAINT notifications_retry_check CHECK (retry_count >= 0 AND retry_count <= max_retries)
);

-- Notification Groups table (for grouping recipients)
CREATE TABLE notification_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    group_type VARCHAR(50) NOT NULL, -- 'department', 'role', 'project', 'escalation'
    is_active BOOLEAN DEFAULT true,
    escalation_delay_minutes INTEGER, -- For escalation groups
    group_metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Group Memberships table (many-to-many between groups and recipients)
CREATE TABLE group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL,
    recipient_id UUID NOT NULL,
    role VARCHAR(100), -- 'member', 'escalation_contact', 'backup'
    is_active BOOLEAN DEFAULT true,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT group_memberships_group_fkey FOREIGN KEY (group_id) REFERENCES notification_groups(id) ON DELETE CASCADE,
    CONSTRAINT group_memberships_recipient_fkey FOREIGN KEY (recipient_id) REFERENCES notification_recipients(id) ON DELETE CASCADE,
    UNIQUE(group_id, recipient_id)
);

-- Notification Delivery Attempts table (for tracking retries and debugging)
CREATE TABLE delivery_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL,
    attempt_number INTEGER NOT NULL,
    channel notification_channel NOT NULL,
    recipient_address TEXT NOT NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL, -- 'success', 'failed', 'timeout', 'rate_limited'
    response_code INTEGER,
    response_message TEXT,
    delivery_time_ms INTEGER,
    provider VARCHAR(100), -- 'sendgrid', 'slack_api', 'twilio', etc.
    provider_message_id VARCHAR(255),
    attempt_metadata JSONB DEFAULT '{}',
    
    CONSTRAINT delivery_attempts_notification_fkey FOREIGN KEY (notification_id) REFERENCES notifications(id) ON DELETE CASCADE,
    CONSTRAINT delivery_attempts_number_check CHECK (attempt_number > 0)
);

-- Notification Statistics table (for analytics and monitoring)
CREATE TABLE notification_statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    hour INTEGER NOT NULL,
    channel notification_channel NOT NULL,
    priority notification_priority NOT NULL,
    status notification_status NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    average_delivery_time_ms INTEGER,
    success_rate DECIMAL(5,2),
    statistics_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT notification_statistics_hour_check CHECK (hour >= 0 AND hour <= 23),
    CONSTRAINT notification_statistics_count_check CHECK (count >= 0),
    CONSTRAINT notification_statistics_success_rate_check CHECK (success_rate >= 0.0 AND success_rate <= 100.0),
    UNIQUE(date, hour, channel, priority, status)
);

-- Channel Configurations table (settings for each notification channel)
CREATE TABLE channel_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel notification_channel NOT NULL UNIQUE,
    provider VARCHAR(100) NOT NULL,
    configuration JSONB NOT NULL, -- Provider-specific config (API keys, endpoints, etc.)
    rate_limit_per_minute INTEGER DEFAULT 60,
    rate_limit_per_hour INTEGER DEFAULT 1000,
    timeout_seconds INTEGER DEFAULT 30,
    retry_intervals INTEGER[] DEFAULT ARRAY[60, 300, 900], -- Retry after N seconds
    is_enabled BOOLEAN DEFAULT true,
    health_check_url TEXT,
    last_health_check TIMESTAMPTZ,
    health_status VARCHAR(20) DEFAULT 'unknown',
    configuration_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT channel_configurations_rate_limits_check CHECK (
        rate_limit_per_minute > 0 AND rate_limit_per_hour > 0 AND 
        rate_limit_per_hour >= rate_limit_per_minute
    )
);

-- Escalation Chains table (for urgent notifications)
CREATE TABLE escalation_chains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    trigger_conditions JSONB NOT NULL, -- When to trigger escalation
    escalation_steps JSONB NOT NULL, -- Array of escalation steps with delays and recipients
    max_escalation_level INTEGER DEFAULT 3,
    is_active BOOLEAN DEFAULT true,
    last_triggered TIMESTAMPTZ,
    total_escalations INTEGER DEFAULT 0,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT escalation_chains_max_level_check CHECK (max_escalation_level > 0)
);

-- Create indexes for performance
CREATE INDEX idx_notification_templates_type ON notification_templates(template_type);
CREATE INDEX idx_notification_templates_active ON notification_templates(is_active);
CREATE INDEX idx_notification_templates_channels_gin ON notification_templates USING gin(supported_channels);

CREATE INDEX idx_notification_recipients_email ON notification_recipients(email);
CREATE INDEX idx_notification_recipients_type ON notification_recipients(type);
CREATE INDEX idx_notification_recipients_active ON notification_recipients(is_active);
CREATE INDEX idx_notification_recipients_channels_gin ON notification_recipients USING gin(preferred_channels);

CREATE INDEX idx_notification_rules_active ON notification_rules(is_active);
CREATE INDEX idx_notification_rules_execution_order ON notification_rules(execution_order);
CREATE INDEX idx_notification_rules_conditions_gin ON notification_rules USING gin(conditions);

CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_channel ON notifications(channel);
CREATE INDEX idx_notifications_scheduled_for ON notifications(scheduled_for);
CREATE INDEX idx_notifications_sent_at ON notifications(sent_at);
CREATE INDEX idx_notifications_correlation_id ON notifications(correlation_id);
CREATE INDEX idx_notifications_source_service ON notifications(source_service);
CREATE INDEX idx_notifications_recipient_id ON notifications(recipient_id);
CREATE INDEX idx_notifications_template_id ON notifications(template_id);

CREATE INDEX idx_notification_groups_active ON notification_groups(is_active);
CREATE INDEX idx_notification_groups_type ON notification_groups(group_type);

CREATE INDEX idx_group_memberships_group_id ON group_memberships(group_id);
CREATE INDEX idx_group_memberships_recipient_id ON group_memberships(recipient_id);
CREATE INDEX idx_group_memberships_active ON group_memberships(is_active);

CREATE INDEX idx_delivery_attempts_notification_id ON delivery_attempts(notification_id);
CREATE INDEX idx_delivery_attempts_attempted_at ON delivery_attempts(attempted_at);
CREATE INDEX idx_delivery_attempts_status ON delivery_attempts(status);
CREATE INDEX idx_delivery_attempts_channel ON delivery_attempts(channel);

CREATE INDEX idx_notification_statistics_date_hour ON notification_statistics(date, hour);
CREATE INDEX idx_notification_statistics_channel ON notification_statistics(channel);
CREATE INDEX idx_notification_statistics_priority ON notification_statistics(priority);

CREATE INDEX idx_channel_configurations_channel ON channel_configurations(channel);
CREATE INDEX idx_channel_configurations_enabled ON channel_configurations(is_enabled);

CREATE INDEX idx_escalation_chains_active ON escalation_chains(is_active);
CREATE INDEX idx_escalation_chains_conditions_gin ON escalation_chains USING gin(trigger_conditions);

-- Create trigger functions
CREATE OR REPLACE FUNCTION update_notification_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER trigger_notification_templates_updated_at
    BEFORE UPDATE ON notification_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

CREATE TRIGGER trigger_notification_recipients_updated_at
    BEFORE UPDATE ON notification_recipients
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

CREATE TRIGGER trigger_notification_rules_updated_at
    BEFORE UPDATE ON notification_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

CREATE TRIGGER trigger_notifications_updated_at
    BEFORE UPDATE ON notifications
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

CREATE TRIGGER trigger_notification_groups_updated_at
    BEFORE UPDATE ON notification_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

CREATE TRIGGER trigger_channel_configurations_updated_at
    BEFORE UPDATE ON channel_configurations
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

CREATE TRIGGER trigger_escalation_chains_updated_at
    BEFORE UPDATE ON escalation_chains
    FOR EACH ROW
    EXECUTE FUNCTION update_notification_updated_at();

-- Insert default notification templates
INSERT INTO notification_templates (name, template_type, subject_template, body_template, supported_channels, default_priority, variables) VALUES
    ('temperature_alert', 'alert', 'Temperature Alert: {{location_name}}', 
     'Temperature in {{location_name}} has {{status}} the threshold.\n\nCurrent: {{current_temp}}°C\nThreshold: {{threshold_temp}}°C\nTime: {{alert_time}}\n\nPlease investigate immediately.',
     ARRAY['email', 'slack', 'sms'], 'high', '["location_name", "status", "current_temp", "threshold_temp", "alert_time"]'),
    
    ('sample_status_update', 'status_update', 'Sample Status Update: {{sample_id}}',
     'Sample {{sample_id}} status has been updated to: {{new_status}}\n\nLocation: {{location}}\nUpdated by: {{updated_by}}\nTime: {{update_time}}',
     ARRAY['email', 'slack'], 'normal', '["sample_id", "new_status", "location", "updated_by", "update_time"]'),
     
    ('maintenance_reminder', 'reminder', 'Maintenance Due: {{equipment_name}}',
     'Equipment maintenance is due:\n\nEquipment: {{equipment_name}}\nLocation: {{location}}\nLast Maintenance: {{last_maintenance}}\nDue Date: {{due_date}}\n\nPlease schedule maintenance.',
     ARRAY['email', 'slack'], 'normal', '["equipment_name", "location", "last_maintenance", "due_date"]'),
     
    ('critical_system_alert', 'alert', 'CRITICAL: {{system_name}} Alert',
     'CRITICAL ALERT for {{system_name}}\n\nIssue: {{issue_description}}\nSeverity: {{severity}}\nTime: {{alert_time}}\n\nIMMEDIATE ACTION REQUIRED',
     ARRAY['email', 'slack', 'sms', 'push'], 'critical', '["system_name", "issue_description", "severity", "alert_time"]');

-- Insert default recipients and groups
INSERT INTO notification_recipients (name, type, email, preferred_channels) VALUES
    ('Lab Administrator', 'role', 'admin@lab.local', ARRAY['email', 'slack']),
    ('Emergency Contact', 'external', 'emergency@lab.local', ARRAY['email', 'sms']),
    ('Maintenance Team', 'group', 'maintenance@lab.local', ARRAY['email', 'slack']);

INSERT INTO notification_groups (name, group_type, description) VALUES
    ('Lab Administrators', 'role', 'All laboratory administrators'),
    ('Emergency Response', 'escalation', 'Emergency response team for critical alerts'),
    ('Maintenance Staff', 'department', 'Equipment maintenance personnel');

-- Insert default channel configurations
INSERT INTO channel_configurations (channel, provider, configuration, rate_limit_per_minute, rate_limit_per_hour) VALUES
    ('email', 'smtp', '{"host": "smtp.lab.local", "port": 587, "use_tls": true}', 30, 500),
    ('slack', 'slack_api', '{"workspace": "lab-workspace", "bot_token": "xoxb-..."}', 100, 2000),
    ('sms', 'twilio', '{"account_sid": "...", "auth_token": "..."}', 10, 100);

-- Insert default notification rules
INSERT INTO notification_rules (name, rule_type, conditions, actions, priority, execution_order) VALUES
    ('Critical Temperature Alerts', 'alert_routing', 
     '{"alert_type": "temperature", "severity": ["high", "critical"]}',
     '{"channels": ["email", "slack", "sms"], "recipients": ["emergency_group"], "immediate": true}',
     'critical', 1),
     
    ('Standard Maintenance Reminders', 'schedule',
     '{"event_type": "maintenance_due", "days_before": 7}',
     '{"channels": ["email"], "recipients": ["maintenance_group"], "schedule": "daily"}',
     'normal', 50);

COMMENT ON TABLE notification_templates IS 'Templates for various types of notifications with variable substitution';
COMMENT ON TABLE notification_recipients IS 'Recipients for notifications (users, groups, external contacts)';
COMMENT ON TABLE notification_rules IS 'Rules for routing and escalating notifications based on conditions';
COMMENT ON TABLE notifications IS 'Individual notification instances sent to recipients';
COMMENT ON TABLE notification_groups IS 'Groups of recipients for organized notification distribution';
COMMENT ON TABLE group_memberships IS 'Many-to-many relationship between groups and recipients';
COMMENT ON TABLE delivery_attempts IS 'Tracking of notification delivery attempts for debugging and analytics';
COMMENT ON TABLE notification_statistics IS 'Aggregated statistics for monitoring notification system performance';
COMMENT ON TABLE channel_configurations IS 'Configuration settings for different notification channels';
COMMENT ON TABLE escalation_chains IS 'Escalation procedures for urgent notifications'; 