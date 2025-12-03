use crate::config::{read_config, CentyConfig};
use crate::migration::{create_registry, MigrationExecutor};
use crate::version::{compare_versions, daemon_version, SemVer, VersionComparison};
use crate::docs::{
    create_doc, delete_doc, get_doc, list_docs, update_doc, CreateDocOptions, UpdateDocOptions,
};
use crate::issue::{
    create_issue, delete_issue, get_issue, get_issue_by_display_number, list_issues, priority_label, update_issue,
    CreateIssueOptions, UpdateIssueOptions,
    // Asset imports
    add_asset, delete_asset as delete_asset_fn, get_asset, list_assets, list_shared_assets,
    AssetInfo, AssetScope,
};
use crate::manifest::{read_manifest, ManagedFileType as InternalFileType};
use crate::reconciliation::{
    build_reconciliation_plan, execute_reconciliation, ReconciliationDecisions,
};
use crate::registry::{
    get_project_info, list_projects, track_project_async, untrack_project, ProjectInfo,
};
use crate::utils::get_centy_path;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
use tracing::info;

// Import generated protobuf types
pub mod proto {
    tonic::include_proto!("centy");
}

use proto::centy_daemon_server::CentyDaemon;
use proto::*;

/// Signal type for daemon shutdown/restart
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownSignal {
    None,
    Shutdown,
    Restart,
}

pub struct CentyDaemonService {
    shutdown_tx: Arc<watch::Sender<ShutdownSignal>>,
    exe_path: Option<PathBuf>,
}

impl CentyDaemonService {
    pub fn new(shutdown_tx: Arc<watch::Sender<ShutdownSignal>>, exe_path: Option<PathBuf>) -> Self {
        Self { shutdown_tx, exe_path }
    }
}

#[tonic::async_trait]
impl CentyDaemon for CentyDaemonService {
    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Convert int32 priority: 0 means use default, otherwise use the value
        let options = CreateIssueOptions {
            title: req.title,
            description: req.description,
            priority: if req.priority == 0 { None } else { Some(req.priority as u32) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            custom_fields: req.custom_fields,
            template: if req.template.is_empty() { None } else { Some(req.template) },
        };

        match create_issue(project_path, options).await {
            #[allow(deprecated)]
            Ok(result) => Ok(Response::new(CreateIssueResponse {
                success: true,
                error: String::new(),
                id: result.id.clone(),
                display_number: result.display_number,
                issue_number: result.issue_number, // Legacy
                created_files: result.created_files,
                manifest: Some(manifest_to_proto(&result.manifest)),
            })),
            Err(e) => Ok(Response::new(CreateIssueResponse {
                success: false,
                error: e.to_string(),
                id: String::new(),
                display_number: 0,
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
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map(|c| c.priority_levels).unwrap_or(3);

        match get_issue(project_path, &req.issue_id).await {
            Ok(issue) => Ok(Response::new(issue_to_proto(&issue, priority_levels))),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn get_issue_by_display_number(
        &self,
        request: Request<GetIssueByDisplayNumberRequest>,
    ) -> Result<Response<Issue>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map(|c| c.priority_levels).unwrap_or(3);

        match get_issue_by_display_number(project_path, req.display_number).await {
            Ok(issue) => Ok(Response::new(issue_to_proto(&issue, priority_levels))),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    async fn list_issues(
        &self,
        request: Request<ListIssuesRequest>,
    ) -> Result<Response<ListIssuesResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map(|c| c.priority_levels).unwrap_or(3);

        let status_filter = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        // Convert int32 priority filter: 0 means no filter
        let priority_filter = if req.priority == 0 { None } else { Some(req.priority as u32) };

        match list_issues(project_path, status_filter, priority_filter).await {
            Ok(issues) => {
                let total_count = issues.len() as i32;
                Ok(Response::new(ListIssuesResponse {
                    issues: issues.into_iter().map(|i| issue_to_proto(&i, priority_levels)).collect(),
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
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        // Read config for priority_levels (for label generation)
        let config = read_config(project_path).await.ok().flatten();
        let priority_levels = config.as_ref().map(|c| c.priority_levels).unwrap_or(3);

        // Convert int32 priority: 0 means don't update, otherwise use the value
        let options = UpdateIssueOptions {
            title: if req.title.is_empty() { None } else { Some(req.title) },
            description: if req.description.is_empty() { None } else { Some(req.description) },
            status: if req.status.is_empty() { None } else { Some(req.status) },
            priority: if req.priority == 0 { None } else { Some(req.priority as u32) },
            custom_fields: req.custom_fields,
        };

        match update_issue(project_path, &req.issue_id, options).await {
            Ok(result) => Ok(Response::new(UpdateIssueResponse {
                success: true,
                error: String::new(),
                issue: Some(issue_to_proto(&result.issue, priority_levels)),
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
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match delete_issue(project_path, &req.issue_id).await {
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match read_config(project_path).await {
            Ok(Some(config)) => Ok(Response::new(config_to_proto(&config))),
            Ok(None) => Ok(Response::new(Config {
                custom_fields: vec![],
                defaults: std::collections::HashMap::new(),
                priority_levels: 3, // Default
                allowed_states: vec![
                    "open".to_string(),
                    "in-progress".to_string(),
                    "closed".to_string(),
                ],
                default_state: "open".to_string(),
                version: crate::utils::CENTY_VERSION.to_string(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn is_initialized(
        &self,
        request: Request<IsInitializedRequest>,
    ) -> Result<Response<IsInitializedResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let options = CreateDocOptions {
            title: req.title,
            content: req.content,
            slug: if req.slug.is_empty() { None } else { Some(req.slug) },
            template: if req.template.is_empty() { None } else { Some(req.template) },
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
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
        track_project_async(req.project_path.clone());
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

    // ============ Asset RPCs ============

    async fn add_asset(
        &self,
        request: Request<AddAssetRequest>,
    ) -> Result<Response<AddAssetResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let scope = if req.is_shared {
            AssetScope::Shared
        } else {
            AssetScope::IssueSpecific
        };

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match add_asset(project_path, issue_id, req.data, &req.filename, scope).await {
            Ok(result) => {
                // Re-read manifest for response
                let manifest = read_manifest(project_path).await.ok().flatten();
                Ok(Response::new(AddAssetResponse {
                    success: true,
                    error: String::new(),
                    asset: Some(asset_info_to_proto(&result.asset)),
                    path: result.path,
                    manifest: manifest.map(|m| manifest_to_proto(&m)),
                }))
            }
            Err(e) => Ok(Response::new(AddAssetResponse {
                success: false,
                error: e.to_string(),
                asset: None,
                path: String::new(),
                manifest: None,
            })),
        }
    }

    async fn list_assets(
        &self,
        request: Request<ListAssetsRequest>,
    ) -> Result<Response<ListAssetsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match list_assets(project_path, &req.issue_id, req.include_shared).await {
            Ok(assets) => {
                let total_count = assets.len() as i32;
                Ok(Response::new(ListAssetsResponse {
                    assets: assets.iter().map(asset_info_to_proto).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_asset(
        &self,
        request: Request<GetAssetRequest>,
    ) -> Result<Response<GetAssetResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match get_asset(project_path, issue_id, &req.filename, req.is_shared).await {
            Ok((data, asset_info)) => Ok(Response::new(GetAssetResponse {
                success: true,
                error: String::new(),
                data,
                asset: Some(asset_info_to_proto(&asset_info)),
            })),
            Err(e) => Ok(Response::new(GetAssetResponse {
                success: false,
                error: e.to_string(),
                data: vec![],
                asset: None,
            })),
        }
    }

    async fn delete_asset(
        &self,
        request: Request<DeleteAssetRequest>,
    ) -> Result<Response<DeleteAssetResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let issue_id = if req.issue_id.is_empty() {
            None
        } else {
            Some(req.issue_id.as_str())
        };

        match delete_asset_fn(project_path, issue_id, &req.filename, req.is_shared).await {
            Ok(result) => {
                // Re-read manifest for response
                let manifest = read_manifest(project_path).await.ok().flatten();
                Ok(Response::new(DeleteAssetResponse {
                    success: true,
                    error: String::new(),
                    filename: result.filename,
                    was_shared: result.was_shared,
                    manifest: manifest.map(|m| manifest_to_proto(&m)),
                }))
            }
            Err(e) => Ok(Response::new(DeleteAssetResponse {
                success: false,
                error: e.to_string(),
                filename: String::new(),
                was_shared: false,
                manifest: None,
            })),
        }
    }

    async fn list_shared_assets(
        &self,
        request: Request<ListSharedAssetsRequest>,
    ) -> Result<Response<ListAssetsResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        match list_shared_assets(project_path).await {
            Ok(assets) => {
                let total_count = assets.len() as i32;
                Ok(Response::new(ListAssetsResponse {
                    assets: assets.iter().map(asset_info_to_proto).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ============ Project Registry RPCs ============

    async fn list_projects(
        &self,
        request: Request<ListProjectsRequest>,
    ) -> Result<Response<ListProjectsResponse>, Status> {
        let req = request.into_inner();

        match list_projects(req.include_stale).await {
            Ok(projects) => {
                let total_count = projects.len() as i32;
                Ok(Response::new(ListProjectsResponse {
                    projects: projects.into_iter().map(|p| project_info_to_proto(&p)).collect(),
                    total_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn register_project(
        &self,
        request: Request<RegisterProjectRequest>,
    ) -> Result<Response<RegisterProjectResponse>, Status> {
        let req = request.into_inner();

        // Track the project (this creates or updates the entry)
        if let Err(e) = crate::registry::track_project(&req.project_path).await {
            return Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: e.to_string(),
                project: None,
            }));
        }

        // Get the project info
        match get_project_info(&req.project_path).await {
            Ok(Some(info)) => Ok(Response::new(RegisterProjectResponse {
                success: true,
                error: String::new(),
                project: Some(project_info_to_proto(&info)),
            })),
            Ok(None) => Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: "Failed to retrieve project after registration".to_string(),
                project: None,
            })),
            Err(e) => Ok(Response::new(RegisterProjectResponse {
                success: false,
                error: e.to_string(),
                project: None,
            })),
        }
    }

    async fn untrack_project(
        &self,
        request: Request<UntrackProjectRequest>,
    ) -> Result<Response<UntrackProjectResponse>, Status> {
        let req = request.into_inner();

        match untrack_project(&req.project_path).await {
            Ok(()) => Ok(Response::new(UntrackProjectResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(UntrackProjectResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_project_info(
        &self,
        request: Request<GetProjectInfoRequest>,
    ) -> Result<Response<GetProjectInfoResponse>, Status> {
        let req = request.into_inner();

        match get_project_info(&req.project_path).await {
            Ok(Some(info)) => Ok(Response::new(GetProjectInfoResponse {
                found: true,
                project: Some(project_info_to_proto(&info)),
            })),
            Ok(None) => Ok(Response::new(GetProjectInfoResponse {
                found: false,
                project: None,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // ============ Version RPCs ============

    async fn get_daemon_info(
        &self,
        _request: Request<GetDaemonInfoRequest>,
    ) -> Result<Response<DaemonInfo>, Status> {
        let daemon_ver = daemon_version();
        let registry = create_registry();

        Ok(Response::new(DaemonInfo {
            version: daemon_ver.to_string(),
            available_versions: registry.available_versions(),
        }))
    }

    async fn get_project_version(
        &self,
        request: Request<GetProjectVersionRequest>,
    ) -> Result<Response<ProjectVersionInfo>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let config = read_config(project_path).await.ok().flatten();
        let project_ver_str = config
            .as_ref()
            .and_then(|c| c.version.clone())
            .unwrap_or_else(|| crate::utils::CENTY_VERSION.to_string());

        let project_ver = match SemVer::parse(&project_ver_str) {
            Ok(v) => v,
            Err(e) => return Err(Status::invalid_argument(e.to_string())),
        };
        let daemon_ver = daemon_version();

        let comparison = compare_versions(&project_ver, &daemon_ver);
        let (comparison_str, degraded) = match comparison {
            VersionComparison::Equal => ("equal", false),
            VersionComparison::ProjectBehind => ("project_behind", false),
            VersionComparison::ProjectAhead => ("project_ahead", true),
        };

        Ok(Response::new(ProjectVersionInfo {
            project_version: project_ver_str,
            daemon_version: daemon_ver.to_string(),
            comparison: comparison_str.to_string(),
            degraded_mode: degraded,
        }))
    }

    async fn update_version(
        &self,
        request: Request<UpdateVersionRequest>,
    ) -> Result<Response<UpdateVersionResponse>, Status> {
        let req = request.into_inner();
        track_project_async(req.project_path.clone());
        let project_path = Path::new(&req.project_path);

        let target = match SemVer::parse(&req.target_version) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(UpdateVersionResponse {
                    success: false,
                    error: format!("Invalid target version: {}", e),
                    from_version: String::new(),
                    to_version: String::new(),
                    migrations_applied: vec![],
                }));
            }
        };

        let registry = create_registry();
        let executor = MigrationExecutor::new(registry);

        match executor.migrate(project_path, &target).await {
            Ok(result) => Ok(Response::new(UpdateVersionResponse {
                success: result.success,
                error: result.error.unwrap_or_default(),
                from_version: result.from_version,
                to_version: result.to_version,
                migrations_applied: result.migrations_applied,
            })),
            Err(e) => Ok(Response::new(UpdateVersionResponse {
                success: false,
                error: e.to_string(),
                from_version: String::new(),
                to_version: String::new(),
                migrations_applied: vec![],
            })),
        }
    }

    // ============ Daemon Control RPCs ============

    async fn shutdown(
        &self,
        request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        let req = request.into_inner();
        let delay = req.delay_seconds;

        info!("Shutdown requested with delay: {} seconds", delay);

        // Clone the sender for use in the spawned task
        let shutdown_tx = self.shutdown_tx.clone();

        // Spawn a task to handle the delayed shutdown
        tokio::spawn(async move {
            if delay > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
            }
            let _ = shutdown_tx.send(ShutdownSignal::Shutdown);
        });

        let message = if delay > 0 {
            format!("Daemon will shutdown in {} seconds", delay)
        } else {
            "Daemon shutting down".to_string()
        };

        Ok(Response::new(ShutdownResponse {
            success: true,
            message,
        }))
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartResponse>, Status> {
        let req = request.into_inner();
        let delay = req.delay_seconds;

        info!("Restart requested with delay: {} seconds", delay);

        // Check if we have the executable path
        let exe_path = match &self.exe_path {
            Some(path) => path.clone(),
            None => {
                return Ok(Response::new(RestartResponse {
                    success: false,
                    message: "Cannot restart: unable to determine executable path".to_string(),
                }));
            }
        };

        // Clone what we need for the spawned task
        let shutdown_tx = self.shutdown_tx.clone();

        // Spawn a task to handle the delayed restart
        tokio::spawn(async move {
            if delay > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
            }

            // Spawn a new daemon process before shutting down
            info!("Spawning new daemon process: {:?}", exe_path);
            match Command::new(&exe_path).spawn() {
                Ok(_) => {
                    info!("New daemon process spawned successfully");
                    // Signal the current server to shutdown
                    let _ = shutdown_tx.send(ShutdownSignal::Restart);
                }
                Err(e) => {
                    info!("Failed to spawn new daemon process: {}", e);
                }
            }
        });

        let message = if delay > 0 {
            format!("Daemon will restart in {} seconds", delay)
        } else {
            "Daemon restarting".to_string()
        };

        Ok(Response::new(RestartResponse {
            success: true,
            message,
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
        priority_levels: config.priority_levels as i32,
        allowed_states: config.allowed_states.clone(),
        default_state: config.default_state.clone(),
        version: config.effective_version(),
    }
}

#[allow(deprecated)]
fn issue_to_proto(issue: &crate::issue::Issue, priority_levels: u32) -> Issue {
    Issue {
        id: issue.id.clone(),
        display_number: issue.metadata.display_number,
        issue_number: issue.issue_number.clone(), // Legacy
        title: issue.title.clone(),
        description: issue.description.clone(),
        metadata: Some(IssueMetadata {
            display_number: issue.metadata.display_number,
            status: issue.metadata.status.clone(),
            priority: issue.metadata.priority as i32,
            created_at: issue.metadata.created_at.clone(),
            updated_at: issue.metadata.updated_at.clone(),
            custom_fields: issue.metadata.custom_fields.clone(),
            priority_label: priority_label(issue.metadata.priority, priority_levels),
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

fn project_info_to_proto(info: &ProjectInfo) -> proto::ProjectInfo {
    proto::ProjectInfo {
        path: info.path.clone(),
        first_accessed: info.first_accessed.clone(),
        last_accessed: info.last_accessed.clone(),
        issue_count: info.issue_count,
        doc_count: info.doc_count,
        initialized: info.initialized,
        name: info.name.clone().unwrap_or_default(),
    }
}

fn asset_info_to_proto(asset: &AssetInfo) -> Asset {
    Asset {
        filename: asset.filename.clone(),
        hash: asset.hash.clone(),
        size: asset.size,
        mime_type: asset.mime_type.clone(),
        is_shared: asset.is_shared,
        created_at: asset.created_at.clone(),
    }
}
