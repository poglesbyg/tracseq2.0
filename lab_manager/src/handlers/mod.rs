pub mod dashboard;
pub mod health;
pub mod rag_proxy;
pub mod reports;
pub mod samples;
pub mod sequencing;
pub mod spreadsheets;
pub mod storage;
pub mod templates;
pub mod users;

pub use dashboard::{get_dashboard_stats, DashboardStats};
pub use health::{
    application_metrics, database_health_check, health_check as basic_health_check, liveness_check,
    readiness_check, system_health_check,
};
pub use rag_proxy::{process_document, query_submissions};
pub use reports::{
    create_custom_report, delete_report, get_available_templates, get_report, list_reports,
    save_report_template, update_report, CreateReportRequest, CustomReport, QueryRequest,
    ReportTemplate, SaveTemplateRequest,
};
pub use samples::{
    create_sample, delete_sample, get_sample, list_samples, update_sample, CreateSampleRequest,
    Sample, UpdateSampleRequest,
};
pub use sequencing::{
    create_sequencing_job, delete_sequencing_job, get_sequencing_job, list_sequencing_jobs,
    update_sequencing_job, CreateSequencingJobRequest, SequencingJob, UpdateSequencingJobRequest,
};
pub use spreadsheets::{
    create_dataset, delete_dataset, get_dataset, list_datasets, search_spreadsheet_data,
    update_dataset, CreateDatasetRequest, DatasetInfo, SearchRequest, UpdateDatasetRequest,
};
pub use storage::{get_storage_locations, update_storage_location, StorageLocationInfo};
pub use templates::{
    create_template, delete_template, get_template, list_templates, update_template,
    CreateTemplateRequest, Template, UpdateTemplateRequest,
};
pub use users::{
    create_user, delete_user, get_current_user, get_user, list_users, login, logout,
    reset_password, update_user,
};
