use crate::config::{read_config, CentyConfig};
use crate::issue::{create_issue, CreateIssueOptions};
use crate::manifest::{read_manifest, ManagedFileType as InternalFileType};
use crate::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use crate::utils::get_centy_path;
use std::collections::HashSet;
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
            Ok(plan) => Ok(Response::new(ReconciliationPlan {
                to_create: plan.to_create.into_iter().map(file_info_to_proto).collect(),
                to_restore: plan.to_restore.into_iter().map(file_info_to_proto).collect(),
                to_reset: plan.to_reset.into_iter().map(file_info_to_proto).collect(),
                up_to_date: plan.up_to_date.into_iter().map(file_info_to_proto).collect(),
                user_files: plan.user_files.into_iter().map(file_info_to_proto).collect(),
                needs_decisions: plan.needs_decisions(),
            })),
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
