use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    // Create QAQC schema if it doesn't exist
    sqlx::query("CREATE SCHEMA IF NOT EXISTS qaqc")
        .execute(pool)
        .await
        .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Create custom types
    create_custom_types(pool).await?;
    
    // Create tables
    create_tables(pool).await?;
    
    // Create indexes
    create_indexes(pool).await?;
    
    Ok(())
}

async fn create_custom_types(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    let type_definitions = vec![
        "CREATE TYPE qc_workflow_status AS ENUM ('draft', 'active', 'executing', 'completed', 'failed', 'cancelled', 'suspended')",
        "CREATE TYPE qc_workflow_type AS ENUM ('samplevalidation', 'sequencingqc', 'dataquality', 'compliancecheck', 'performanceanalysis', 'libraryqc', 'spreadsheetvalidation', 'custom')",
        "CREATE TYPE qc_step_type AS ENUM ('validation', 'measurement', 'analysis', 'comparison', 'approval', 'documentation', 'notification', 'integration', 'customscript')",
        "CREATE TYPE qc_trigger_type AS ENUM ('scheduled', 'eventbased', 'thresholdbased', 'statuschange', 'dataavailable', 'manual')",
        "CREATE TYPE threshold_severity AS ENUM ('critical', 'warning', 'info', 'advisory')",
        "CREATE TYPE quality_check_type AS ENUM ('numericrange', 'stringmatch', 'regexmatch', 'fileexists', 'databasequery', 'servicecall', 'statistical', 'visual', 'custom')",
        "CREATE TYPE comparison_operator AS ENUM ('equal', 'notequal', 'greaterthan', 'lessthan', 'greaterthanorequal', 'lessthanorequal', 'contains', 'startswith', 'endswith', 'regex', 'in', 'notin')",
        "CREATE TYPE logic_operator AS ENUM ('and', 'or', 'not')",
        "CREATE TYPE quality_metric_type AS ENUM ('concentration', 'purity', 'integrity', 'yield', 'coverage', 'quality', 'error', 'performance', 'compliance', 'custom')",
        "CREATE TYPE metric_status AS ENUM ('pass', 'fail', 'warning', 'pending', 'notapplicable')",
        "CREATE TYPE compliance_rule_type AS ENUM ('validation', 'documentation', 'approval', 'retention', 'access', 'audit', 'reporting', 'custom')",
        "CREATE TYPE compliance_severity AS ENUM ('critical', 'high', 'medium', 'low', 'info')",
        "CREATE TYPE trigger_action_type AS ENUM ('startworkflow', 'sendnotification', 'updatestatus', 'createtask', 'logevent', 'callservice', 'custom')",
        "CREATE TYPE compliance_action_type AS ENUM ('require', 'validate', 'document', 'approve', 'archive', 'notify', 'audit', 'custom')",
        "CREATE TYPE report_type AS ENUM ('quality', 'compliance', 'performance', 'trend', 'summary', 'custom')",
        "CREATE TYPE period_type AS ENUM ('hour', 'day', 'week', 'month', 'quarter', 'year', 'custom')",
        "CREATE TYPE trend_direction AS ENUM ('increasing', 'decreasing', 'stable', 'volatile', 'unknown')",
        "CREATE TYPE execution_priority AS ENUM ('critical', 'high', 'normal', 'low')",
    ];

    for type_def in type_definitions {
        sqlx::query(&format!("{} IF NOT EXISTS", type_def))
            .execute(pool)
            .await
            .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;
    }

    Ok(())
}

async fn create_tables(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    // QC Workflows table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.qc_workflows (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            name VARCHAR(255) NOT NULL,
            description TEXT,
            workflow_type qc_workflow_type NOT NULL,
            status qc_workflow_status NOT NULL DEFAULT 'draft',
            steps JSONB NOT NULL DEFAULT '[]',
            triggers JSONB NOT NULL DEFAULT '[]',
            quality_thresholds JSONB NOT NULL DEFAULT '{}',
            compliance_requirements JSONB NOT NULL DEFAULT '[]',
            version INTEGER NOT NULL DEFAULT 1,
            created_by UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_executed TIMESTAMPTZ
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Quality Metrics table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.quality_metrics (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            name VARCHAR(255) NOT NULL,
            metric_type quality_metric_type NOT NULL,
            value DECIMAL NOT NULL,
            unit VARCHAR(50),
            sample_id UUID,
            workflow_id UUID REFERENCES qaqc.qc_workflows(id),
            step_id VARCHAR(255),
            threshold_id UUID,
            status metric_status NOT NULL,
            measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            measured_by UUID,
            metadata JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Compliance Rules table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.compliance_rules (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            name VARCHAR(255) NOT NULL,
            description TEXT,
            rule_type compliance_rule_type NOT NULL,
            standard VARCHAR(100) NOT NULL,
            section VARCHAR(100),
            severity compliance_severity NOT NULL,
            conditions JSONB NOT NULL DEFAULT '[]',
            actions JSONB NOT NULL DEFAULT '[]',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Workflow Executions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.workflow_executions (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            workflow_id UUID NOT NULL REFERENCES qaqc.qc_workflows(id),
            target_id UUID NOT NULL,
            target_type VARCHAR(100) NOT NULL,
            status qc_workflow_status NOT NULL DEFAULT 'executing',
            priority execution_priority NOT NULL DEFAULT 'normal',
            parameters JSONB NOT NULL DEFAULT '{}',
            results JSONB NOT NULL DEFAULT '{}',
            error_details TEXT,
            started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ,
            executed_by UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Quality Analysis Reports table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.quality_reports (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            report_type report_type NOT NULL,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            time_period_start TIMESTAMPTZ NOT NULL,
            time_period_end TIMESTAMPTZ NOT NULL,
            period_type period_type NOT NULL,
            metrics JSONB NOT NULL DEFAULT '[]',
            trends JSONB NOT NULL DEFAULT '[]',
            recommendations JSONB NOT NULL DEFAULT '[]',
            generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            generated_by UUID NOT NULL
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Quality Thresholds table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.quality_thresholds (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            metric_name VARCHAR(255) NOT NULL,
            min_value DECIMAL,
            max_value DECIMAL,
            target_value DECIMAL,
            tolerance DECIMAL,
            severity threshold_severity NOT NULL,
            unit VARCHAR(50),
            description TEXT,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    // Audit Trail table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS qaqc.audit_trail (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            entity_type VARCHAR(100) NOT NULL,
            entity_id UUID NOT NULL,
            action VARCHAR(100) NOT NULL,
            old_values JSONB,
            new_values JSONB,
            user_id UUID NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            ip_address INET,
            user_agent TEXT
        )
    "#)
    .execute(pool)
    .await
    .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;

    Ok(())
}

async fn create_indexes(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_qc_workflows_status ON qaqc.qc_workflows(status)",
        "CREATE INDEX IF NOT EXISTS idx_qc_workflows_type ON qaqc.qc_workflows(workflow_type)",
        "CREATE INDEX IF NOT EXISTS idx_qc_workflows_created_at ON qaqc.qc_workflows(created_at)",
        "CREATE INDEX IF NOT EXISTS idx_quality_metrics_type ON qaqc.quality_metrics(metric_type)",
        "CREATE INDEX IF NOT EXISTS idx_quality_metrics_status ON qaqc.quality_metrics(status)",
        "CREATE INDEX IF NOT EXISTS idx_quality_metrics_sample_id ON qaqc.quality_metrics(sample_id)",
        "CREATE INDEX IF NOT EXISTS idx_quality_metrics_workflow_id ON qaqc.quality_metrics(workflow_id)",
        "CREATE INDEX IF NOT EXISTS idx_quality_metrics_measured_at ON qaqc.quality_metrics(measured_at)",
        "CREATE INDEX IF NOT EXISTS idx_compliance_rules_standard ON qaqc.compliance_rules(standard)",
        "CREATE INDEX IF NOT EXISTS idx_compliance_rules_rule_type ON qaqc.compliance_rules(rule_type)",
        "CREATE INDEX IF NOT EXISTS idx_compliance_rules_active ON qaqc.compliance_rules(active)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id ON qaqc.workflow_executions(workflow_id)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_target_id ON qaqc.workflow_executions(target_id)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON qaqc.workflow_executions(status)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_executions_started_at ON qaqc.workflow_executions(started_at)",
        "CREATE INDEX IF NOT EXISTS idx_quality_reports_report_type ON qaqc.quality_reports(report_type)",
        "CREATE INDEX IF NOT EXISTS idx_quality_reports_generated_at ON qaqc.quality_reports(generated_at)",
        "CREATE INDEX IF NOT EXISTS idx_quality_thresholds_metric_name ON qaqc.quality_thresholds(metric_name)",
        "CREATE INDEX IF NOT EXISTS idx_quality_thresholds_active ON qaqc.quality_thresholds(active)",
        "CREATE INDEX IF NOT EXISTS idx_audit_trail_entity_type ON qaqc.audit_trail(entity_type)",
        "CREATE INDEX IF NOT EXISTS idx_audit_trail_entity_id ON qaqc.audit_trail(entity_id)",
        "CREATE INDEX IF NOT EXISTS idx_audit_trail_timestamp ON qaqc.audit_trail(timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_audit_trail_user_id ON qaqc.audit_trail(user_id)",
    ];

    for index in indexes {
        sqlx::query(index)
            .execute(pool)
            .await
            .map_err(|e| sqlx::migrate::MigrateError::Source(Box::new(e)))?;
    }

    Ok(())
} 
