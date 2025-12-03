pub mod create;
pub mod crud;
pub mod id;
mod metadata;
pub mod priority;
pub mod reconcile;
pub mod status;

#[allow(deprecated)]
pub use create::{create_issue, get_next_issue_number, CreateIssueOptions, CreateIssueResult, IssueError};
pub use crud::{
    delete_issue, get_issue, get_issue_by_display_number, list_issues, update_issue,
    DeleteIssueResult, Issue, IssueCrudError, IssueMetadataFlat, UpdateIssueOptions, UpdateIssueResult,
};
pub use id::{generate_issue_id, is_legacy_number, is_uuid, is_valid_issue_folder, short_id};
pub use metadata::IssueMetadata;
pub use priority::{
    default_priority, label_to_priority, migrate_string_priority, priority_label,
    validate_priority, PriorityError,
};
pub use reconcile::{get_next_display_number, reconcile_display_numbers, ReconcileError};
pub use status::validate_status;
