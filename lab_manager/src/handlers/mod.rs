pub mod dashboard;
pub mod rag_proxy;
pub mod reports;
pub mod samples;
pub mod sequencing;
pub mod spreadsheets;
pub mod storage;
pub mod templates;
pub mod users;

pub use dashboard::{get_dashboard_stats, health_check, DashboardStats, HealthStatus};
pub use rag_proxy::{get_rag_health, get_rag_stats, get_rag_submissions, process_rag_document};
pub use reports::{execute_report, get_report_templates, get_schema};
pub use samples::{
    create_sample, create_samples_batch, create_samples_from_rag_data, get_rag_system_status,
    get_sample, list_samples, preview_document_extraction, process_document_and_create_samples,
    query_submission_information, update_sample, validate_sample,
};
pub use sequencing::{
    create_sequencing_job, get_sequencing_job, list_sequencing_jobs, update_job_status,
};
pub use spreadsheets::{
    analyze_column, analyze_dataset, delete_dataset, get_available_filters, get_dataset,
    get_sheet_names, health_check as spreadsheets_health_check, list_datasets, search_data,
    supported_types, upload_spreadsheet, upload_spreadsheet_multiple_sheets,
};
pub use storage::{
    create_storage_location, get_capacity_overview, list_storage_locations, move_sample,
    remove_sample, scan_sample_barcode, store_sample,
};
pub use templates::{
    delete_template, get_template, get_template_data, list_templates, update_template,
    upload_template,
};
pub use users::{
    change_password, confirm_password_reset, create_user, delete_user, get_current_user, get_user,
    get_user_sessions, list_users, login, logout, request_password_reset, revoke_all_sessions,
    revoke_session, shibboleth_login_redirect, shibboleth_logout_redirect, update_current_user,
    update_user,
};
