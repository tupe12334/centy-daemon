use crate::config::{read_config, CentyConfig};
use crate::docs::{
    create_doc, delete_doc, get_doc, list_docs, update_doc,
    CreateDocOptions, UpdateDocOptions,
};
use crate::issue::{
    create_issue, delete_issue, get_issue, list_issues, update_issue,
    CreateIssueOptions, UpdateIssueOptions,
};
use crate::manifest::{read_manifest, ManagedFileType as InternalFileType};
use crate::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use crate::utils::get_centy_path;
use std::path::Path;
use tonic::{Request, Response, Status};

// Import generated protobuf types
pub mod proto {
    tonic::include_proto!("centy");
}

use proto::centy_daemon_server::CentyDaemon;
use proto::*;

pub struct CentyDaemonService;

impl CentyDaemonService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CentyDaemonService {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl CentyDaemon for CentyDaemonService {
    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let decisions = req.decisions.map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        }).unwrap_or_default();

        match execute_reconciliation(project_path, decisions, req.force).await {
            Ok(result) => Ok(Response::new(InitResponse {
                success: true,
                error: String::new(),
                created: result.created,
                restored: result.restored,
                reset: result.reset,
                skipped: result.skipped,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(InitResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
            })),
        }
    }

    async fn get_reconciliation_plan(
        &self,
        request: Request<GetReconciliationPlanRequest>,
    ) -> Result<Response<ReconciliationPlan>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match build_reconciliation_plan(project_path).await {
            Ok(plan) => {
                let needs_decisions = plan.needs_decisions();
                Ok(Response::new(ReconciliationPlan {
                    to_create: plan.to_create.into_iter().map(file_info_to_proto).collect(),
                    to_restore: plan.to_restore.into_iter().map(file_info_to_proto).collect(),
                    to_reset: plan.to_reset.into_iter().map(file_info_to_proto).collect(),
                    up_to_date: plan.up_to_date.into_iter().map(file_info_to_proto).collect(),
                    user_files: plan.user_files.into_iter().map(file_info_to_proto).collect(),
                    needs_decisions,
                }))
            },
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn execute_reconciliation(
        &self,
        request: Request<ExecuteReconciliationRequest>,
    ) -> Result<Response<InitResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let decisions = req.decisions.map(|d| ReconciliationDecisions {
            restore: d.restore.into_iter().collect(),
            reset: d.reset.into_iter().collect(),
        }).unwrap_or_default();

        match execute_reconciliation(project_path, decisions, false).await {
            Ok(result) => Ok(Response::new(InitResponse {
                success: true,
                error: String::new(),
                created: result.created,
                restored: result.restored,
                reset: result.reset,
                skipped: result.skipped,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(InitResponse {
                success: false,
                error: e.to_string(),
                created: vec![],
                restored: vec![],
                reset: vec![],
                skipped: vec![],
                manifest: None,
            })),
        }
    }

    async fn create_issue(
        &self,
        request: Request<CreateIssueRequest>,
    ) -> Result<Response<CreateIssueResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let options = CreateIssueOptions {
            title: req.title,
            description: req.description,
            priority: if req.priority.is_empty() { None } else { Some(req.priority) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            custom_fields: req.custom_fields,
        };

        match create_issue(project_path, options).await {
            Ok(result) => Ok(Response::new(CreateIssueResponse {
                success: true,
                error: String::new(),
                issue_number: result.issue_number,
                created_files: result.created_files,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(CreateIssueResponse {
                success: false,
                error: e.to_string(),
                issue_number: String::new(),
                created_files: vec![],
                manifest: None,
            })),
        }
    }

    async fn get_issue(
        &self,
        request: Request<GetIssueRequest>,
    ) -> Result<Response<Issue>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match get_issue(project_path, &req.issue_number).await {
            Ok(issue) => Ok(Response::new(issue_to_proto(&issue))),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn list_issues(
        &self,
        request: Request<ListIssuesRequest>,
    ) -> Result<Response<ListIssuesResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let status_filter = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let priority_filter = if req.priority.is_empty() { None } else { Some(req.priority.as_str()) };

        match list_issues(project_path, status_filter, priority_filter).await {
            Ok(issues) => {
                let total_count = issues.len() as i32;
                Ok(Response::new(ListIssuesResponse {
                    issues: issues.into_iter().map(|i| issue_to_proto(&i)).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_issue(
        &self,
        request: Request<UpdateIssueRequest>,
    ) -> Result<Response<UpdateIssueResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let options = UpdateIssueOptions {
            title: if req.title.is_empty() { None } else { Some(req.title) },
            description: if req.description.is_empty() { None } else { Some(req.description) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            priority: if req.priority.is_empty() { None } else { Some(req.priority) },
            custom_fields: req.custom_fields,
        };

        match update_issue(project_path, &req.issue_number, options).await {
            Ok(result) => Ok(Response::new(UpdateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(UpdateIssueResponse {
                success: false,
                error: e.to_string(),
                issue: None,
                manifest: None,
            })),
        }
    }

    async fn delete_issue(
        &self,
        request: Request<DeleteIssueRequest>,
    ) -> Result<Response<DeleteIssueResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match delete_issue(project_path, &req.issue_number).await {
            Ok(result) => Ok(Response::new(DeleteIssueResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DeleteIssueResponse {
                success: false,
                error: e.to_string(),
                manifest: None,
            })),
        }
    }

    async fn get_next_issue_number(
        &self,
        request: Request<GetNextIssueNumberRequest>,
    ) -> Result<Response<GetNextIssueNumberResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);
        let issues_path = get_centy_path(project_path).join("issues");

        match crate::issue::create::get_next_issue_number(&issues_path).await {
            Ok(issue_number) => Ok(Response::new(GetNextIssueNumberResponse { issue_number })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_manifest(
        &self,
        request: Request<GetManifestRequest>,
    ) -> Result<Response<Manifest>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match read_manifest(project_path).await {
            Ok(Some(manifest)) => Ok(Response::new(manifest_to_proto(&manifest))),
            Ok(None) => Err(Status::not_found("Manifest not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<Config>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match read_config(project_path).await {
            Ok(Some(config)) => Ok(Response::new(config_to_proto(&config))),
            Ok(None) => Ok(Response::new(Config {
                custom_fields: vec![],
                defaults: std::collections::HashMap::new(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn is_initialized(
        &self,
        request: Request<IsInitializedRequest>,
    ) -> Result<Response<IsInitializedResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);
        let centy_path = get_centy_path(project_path);
        let manifest_path = centy_path.join(".centy-manifest.json");

        let initialized = manifest_path.exists();
        let centy_path_str = if initialized {
            centy_path.to_string_lossy().to_string()
        } else {
            String::new()
        };

        Ok(Response::new(IsInitializedResponse {
            initialized,
            centy_path: centy_path_str,
        }))
    }

    // ============ Doc RPCs ============

    async fn create_doc(
        &self,
        request: Request<CreateDocRequest>,
    ) -> Result<Response<CreateDocResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let options = CreateDocOptions {
            title: req.title,
            content: req.content,
            slug: if req.slug.is_empty() { None } else { Some(req.slug) },
        };

        match create_doc(project_path, options).await {
            Ok(result) => Ok(Response::new(CreateDocResponse {
                success: true,
                error: String::new(),
                slug: result.slug,
                created_file: result.created_file,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(CreateDocResponse {
                success: false,
                error: e.to_string(),
                slug: String::new(),
                created_file: String::new(),
                manifest: None,
            })),
        }
    }

    async fn get_doc(
        &self,
        request: Request<GetDocRequest>,
    ) -> Result<Response<Doc>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match get_doc(project_path, &req.slug).await {
            Ok(doc) => Ok(Response::new(doc_to_proto(&doc))),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn list_docs(
        &self,
        request: Request<ListDocsRequest>,
    ) -> Result<Response<ListDocsResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match list_docs(project_path).await {
            Ok(docs) => {
                let total_count = docs.len() as i32;
                Ok(Response::new(ListDocsResponse {
                    docs: docs.into_iter().map(|d| doc_to_proto(&d)).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_doc(
        &self,
        request: Request<UpdateDocRequest>,
    ) -> Result<Response<UpdateDocResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        let options = UpdateDocOptions {
            title: if req.title.is_empty() { None } else { Some(req.title) },
            content: if req.content.is_empty() { None } else { Some(req.content) },
            new_slug: if req.new_slug.is_empty() { None } else { Some(req.new_slug) },
        };

        match update_doc(project_path, &req.slug, options).await {
            Ok(result) => Ok(Response::new(UpdateDocResponse {
                success: true,
                error: String::new(),
                doc: Some(doc_to_proto(&result.doc)),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(UpdateDocResponse {
                success: false,
                error: e.to_string(),
                doc: None,
                manifest: None,
            })),
        }
    }

    async fn delete_doc(
        &self,
        request: Request<DeleteDocRequest>,
    ) -> Result<Response<DeleteDocResponse>, Status> {
        let req = request.into_inner();
        let project_path = Path::new(&req.project_path);

        match delete_doc(project_path, &req.slug).await {
            Ok(result) => Ok(Response::new(DeleteDocResponse {
                success: true,
                error: String::new(),
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(DeleteDocResponse {
                success: false,
                error: e.to_string(),
                manifest: None,
            })),
        }
    }
}

// Helper functions for converting internal types to proto types

fn manifest_to_proto(manifest: &crate::manifest::CentyManifest) -> Manifest {
    Manifest {
        schema_version: manifest.schema_version as i32,
        centy_version: manifest.centy_version.clone(),
        created_at: manifest.created_at.clone(),
        updated_at: manifest.updated_at.clone(),
        managed_files: manifest
            .managed_files
            .iter()
            .map(|f| ManagedFile {
                path: f.path.clone(),
                hash: f.hash.clone(),
                version: f.version.clone(),
                created_at: f.created_at.clone(),
                file_type: match f.file_type {
                    InternalFileType::File => FileType::File as i32,
                    InternalFileType::Directory => FileType::Directory as i32,
                },
            })
            .collect(),
    }
}

fn file_info_to_proto(info: crate::reconciliation::FileInfo) -> FileInfo {
    FileInfo {
        path: info.path,
        file_type: match info.file_type {
            InternalFileType::File => FileType::File as i32,
            InternalFileType::Directory => FileType::Directory as i32,
        },
        hash: info.hash,
        content_preview: info.content_preview.unwrap_or_default(),
    }
}

fn config_to_proto(config: &CentyConfig) -> Config {
    Config {
        custom_fields: config
            .custom_fields
            .iter()
            .map(|f| CustomFieldDefinition {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                required: f.required,
                default_value: f.default_value.clone().unwrap_or_default(),
                enum_values: f.enum_values.clone(),
            })
            .collect(),
        defaults: config.defaults.clone(),
    }
}

fn issue_to_proto(issue: &crate::issue::Issue) -> Issue {
    Issue {
        issue_number: issue.issue_number.clone(),
        title: issue.title.clone(),
        description: issue.description.clone(),
        metadata: Some(IssueMetadata {
            status: issue.metadata.status.clone(),
            priority: issue.metadata.priority.clone(),
            created_at: issue.metadata.created_at.clone(),
            updated_at: issue.metadata.updated_at.clone(),
            custom_fields: issue.metadata.custom_fields.clone(),
        }),
    }
}

fn doc_to_proto(doc: &crate::docs::Doc) -> Doc {
    Doc {
        slug: doc.slug.clone(),
        title: doc.title.clone(),
        content: doc.content.clone(),
        metadata: Some(DocMetadata {
            created_at: doc.metadata.created_at.clone(),
            updated_at: doc.metadata.updated_at.clone(),
        }),
    }
}
