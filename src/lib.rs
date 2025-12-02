pub mod config;
pub mod docs;
pub mod issue;
pub mod manifest;
pub mod reconciliation;
pub mod registry;
pub mod server;
pub mod template;
pub mod utils;

// Re-export commonly used types
pub use config::{CentyConfig, CustomFieldDefinition};
pub use docs::{
    create_doc, delete_doc, get_doc, list_docs, update_doc,
    CreateDocOptions, CreateDocResult, DeleteDocResult, Doc, DocError, DocMetadata,
    UpdateDocOptions, UpdateDocResult,
};
pub use issue::{
    create_issue, delete_issue, get_issue, list_issues, update_issue,
    CreateIssueOptions, CreateIssueResult, DeleteIssueResult, Issue,
    IssueMetadataFlat, UpdateIssueOptions, UpdateIssueResult,
};
pub use manifest::{CentyManifest, ManagedFile, ManagedFileType};
pub use reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions, ReconciliationPlan,
    ReconciliationResult,
};
pub use registry::{
    get_project_info, list_projects, track_project, untrack_project, ProjectInfo, ProjectRegistry,
    RegistryError, TrackedProject,
};
pub use server::CentyDaemonService;
pub use template::{DocTemplateContext, IssueTemplateContext, TemplateEngine, TemplateError, TemplateType};
