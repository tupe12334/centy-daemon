import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { join } from 'node:path';

// Proto path is relative to the e2e directory - go up one level to find proto/
const PROTO_PATH = join(process.cwd(), '../proto/centy.proto');

// Load proto definition
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: false,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});

const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;

// Type definitions for gRPC client methods
export interface CentyClient {
  // Init
  init(
    request: InitRequest,
    callback: (error: grpc.ServiceError | null, response: InitResponse) => void
  ): void;
  getReconciliationPlan(
    request: GetReconciliationPlanRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ReconciliationPlan
    ) => void
  ): void;
  executeReconciliation(
    request: ExecuteReconciliationRequest,
    callback: (error: grpc.ServiceError | null, response: InitResponse) => void
  ): void;
  isInitialized(
    request: IsInitializedRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: IsInitializedResponse
    ) => void
  ): void;

  // Issues
  createIssue(
    request: CreateIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateIssueResponse
    ) => void
  ): void;
  getIssue(
    request: GetIssueRequest,
    callback: (error: grpc.ServiceError | null, response: Issue) => void
  ): void;
  getIssueByDisplayNumber(
    request: GetIssueByDisplayNumberRequest,
    callback: (error: grpc.ServiceError | null, response: Issue) => void
  ): void;
  listIssues(
    request: ListIssuesRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListIssuesResponse
    ) => void
  ): void;
  updateIssue(
    request: UpdateIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateIssueResponse
    ) => void
  ): void;
  deleteIssue(
    request: DeleteIssueRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteIssueResponse
    ) => void
  ): void;
  getNextIssueNumber(
    request: GetNextIssueNumberRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetNextIssueNumberResponse
    ) => void
  ): void;

  // Docs
  createDoc(
    request: CreateDocRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: CreateDocResponse
    ) => void
  ): void;
  getDoc(
    request: GetDocRequest,
    callback: (error: grpc.ServiceError | null, response: Doc) => void
  ): void;
  listDocs(
    request: ListDocsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListDocsResponse
    ) => void
  ): void;
  updateDoc(
    request: UpdateDocRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateDocResponse
    ) => void
  ): void;
  deleteDoc(
    request: DeleteDocRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteDocResponse
    ) => void
  ): void;

  // Assets
  addAsset(
    request: AddAssetRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: AddAssetResponse
    ) => void
  ): void;
  listAssets(
    request: ListAssetsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListAssetsResponse
    ) => void
  ): void;
  getAsset(
    request: GetAssetRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetAssetResponse
    ) => void
  ): void;
  deleteAsset(
    request: DeleteAssetRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: DeleteAssetResponse
    ) => void
  ): void;
  listSharedAssets(
    request: ListSharedAssetsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListAssetsResponse
    ) => void
  ): void;

  // Projects
  listProjects(
    request: ListProjectsRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ListProjectsResponse
    ) => void
  ): void;
  registerProject(
    request: RegisterProjectRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: RegisterProjectResponse
    ) => void
  ): void;
  untrackProject(
    request: UntrackProjectRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UntrackProjectResponse
    ) => void
  ): void;
  getProjectInfo(
    request: GetProjectInfoRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: GetProjectInfoResponse
    ) => void
  ): void;

  // Config
  getConfig(
    request: GetConfigRequest,
    callback: (error: grpc.ServiceError | null, response: Config) => void
  ): void;
  updateConfig(
    request: UpdateConfigRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateConfigResponse
    ) => void
  ): void;
  getManifest(
    request: GetManifestRequest,
    callback: (error: grpc.ServiceError | null, response: Manifest) => void
  ): void;

  // Daemon control
  getDaemonInfo(
    request: GetDaemonInfoRequest,
    callback: (error: grpc.ServiceError | null, response: DaemonInfo) => void
  ): void;
  getProjectVersion(
    request: GetProjectVersionRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ProjectVersionInfo
    ) => void
  ): void;
  updateVersion(
    request: UpdateVersionRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: UpdateVersionResponse
    ) => void
  ): void;
  shutdown(
    request: ShutdownRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: ShutdownResponse
    ) => void
  ): void;
  restart(
    request: RestartRequest,
    callback: (
      error: grpc.ServiceError | null,
      response: RestartResponse
    ) => void
  ): void;

  // For cleanup
  close(): void;
}

// Request/Response types
export interface InitRequest {
  projectPath: string;
  force?: boolean;
  decisions?: ReconciliationDecisions;
}

export interface InitResponse {
  success: boolean;
  error: string;
  created: string[];
  restored: string[];
  reset: string[];
  skipped: string[];
  manifest?: Manifest;
}

export interface GetReconciliationPlanRequest {
  projectPath: string;
}

export interface ReconciliationPlan {
  toCreate: FileInfo[];
  toRestore: FileInfo[];
  toReset: FileInfo[];
  upToDate: FileInfo[];
  userFiles: FileInfo[];
  needsDecisions: boolean;
}

export interface ExecuteReconciliationRequest {
  projectPath: string;
  decisions: ReconciliationDecisions;
}

export interface ReconciliationDecisions {
  restore: string[];
  reset: string[];
}

export interface IsInitializedRequest {
  projectPath: string;
}

export interface IsInitializedResponse {
  initialized: boolean;
  centyPath: string;
}

export interface CreateIssueRequest {
  projectPath: string;
  title: string;
  description?: string;
  priority?: number;
  status?: string;
  customFields?: Record<string, string>;
  template?: string;
}

export interface CreateIssueResponse {
  success: boolean;
  error: string;
  id: string;
  displayNumber: number;
  issueNumber: string;
  createdFiles: string[];
  manifest?: Manifest;
}

export interface GetIssueRequest {
  projectPath: string;
  issueId: string;
}

export interface GetIssueByDisplayNumberRequest {
  projectPath: string;
  displayNumber: number;
}

export interface Issue {
  id: string;
  displayNumber: number;
  issueNumber: string;
  title: string;
  description: string;
  metadata: IssueMetadata;
}

export interface IssueMetadata {
  displayNumber: number;
  status: string;
  priority: number;
  createdAt: string;
  updatedAt: string;
  customFields: Record<string, string>;
  priorityLabel: string;
}

export interface ListIssuesRequest {
  projectPath: string;
  status?: string;
  priority?: number;
}

export interface ListIssuesResponse {
  issues: Issue[];
  totalCount: number;
}

export interface UpdateIssueRequest {
  projectPath: string;
  issueId: string;
  title?: string;
  description?: string;
  status?: string;
  priority?: number;
  customFields?: Record<string, string>;
}

export interface UpdateIssueResponse {
  success: boolean;
  error: string;
  issue?: Issue;
  manifest?: Manifest;
}

export interface DeleteIssueRequest {
  projectPath: string;
  issueId: string;
}

export interface DeleteIssueResponse {
  success: boolean;
  error: string;
  manifest?: Manifest;
}

export interface GetNextIssueNumberRequest {
  projectPath: string;
}

export interface GetNextIssueNumberResponse {
  issueNumber: string;
}

export interface CreateDocRequest {
  projectPath: string;
  title: string;
  content?: string;
  slug?: string;
  template?: string;
}

export interface CreateDocResponse {
  success: boolean;
  error: string;
  slug: string;
  createdFile: string;
  manifest?: Manifest;
}

export interface GetDocRequest {
  projectPath: string;
  slug: string;
}

export interface Doc {
  slug: string;
  title: string;
  content: string;
  metadata: DocMetadata;
}

export interface DocMetadata {
  createdAt: string;
  updatedAt: string;
}

export interface ListDocsRequest {
  projectPath: string;
}

export interface ListDocsResponse {
  docs: Doc[];
  totalCount: number;
}

export interface UpdateDocRequest {
  projectPath: string;
  slug: string;
  title?: string;
  content?: string;
  newSlug?: string;
}

export interface UpdateDocResponse {
  success: boolean;
  error: string;
  doc?: Doc;
  manifest?: Manifest;
}

export interface DeleteDocRequest {
  projectPath: string;
  slug: string;
}

export interface DeleteDocResponse {
  success: boolean;
  error: string;
  manifest?: Manifest;
}

export interface AddAssetRequest {
  projectPath: string;
  issueId?: string;
  filename: string;
  data: Buffer;
  isShared?: boolean;
}

export interface AddAssetResponse {
  success: boolean;
  error: string;
  asset?: Asset;
  path: string;
  manifest?: Manifest;
}

export interface Asset {
  filename: string;
  hash: string;
  size: number;
  mimeType: string;
  isShared: boolean;
  createdAt: string;
}

export interface ListAssetsRequest {
  projectPath: string;
  issueId?: string;
  includeShared?: boolean;
}

export interface ListAssetsResponse {
  assets: Asset[];
  totalCount: number;
}

export interface GetAssetRequest {
  projectPath: string;
  issueId?: string;
  filename: string;
  isShared?: boolean;
}

export interface GetAssetResponse {
  success: boolean;
  error: string;
  data: Buffer;
  asset?: Asset;
}

export interface DeleteAssetRequest {
  projectPath: string;
  issueId?: string;
  filename: string;
  isShared?: boolean;
}

export interface DeleteAssetResponse {
  success: boolean;
  error: string;
  filename: string;
  wasShared: boolean;
  manifest?: Manifest;
}

export interface ListSharedAssetsRequest {
  projectPath: string;
}

export interface ListProjectsRequest {
  includeStale?: boolean;
}

export interface ListProjectsResponse {
  projects: ProjectInfo[];
  totalCount: number;
}

export interface ProjectInfo {
  path: string;
  firstAccessed: string;
  lastAccessed: string;
  issueCount: number;
  docCount: number;
  initialized: boolean;
  name: string;
}

export interface RegisterProjectRequest {
  projectPath: string;
}

export interface RegisterProjectResponse {
  success: boolean;
  error: string;
  project?: ProjectInfo;
}

export interface UntrackProjectRequest {
  projectPath: string;
}

export interface UntrackProjectResponse {
  success: boolean;
  error: string;
}

export interface GetProjectInfoRequest {
  projectPath: string;
}

export interface GetProjectInfoResponse {
  found: boolean;
  project?: ProjectInfo;
}

export interface GetConfigRequest {
  projectPath: string;
}

export interface Config {
  customFields: CustomFieldDefinition[];
  defaults: Record<string, string>;
  priorityLevels: number;
  allowedStates: string[];
  defaultState: string;
  version: string;
  stateColors: Record<string, string>;
  priorityColors: Record<string, string>;
  llm?: LlmConfig;
}

export interface CustomFieldDefinition {
  name: string;
  fieldType: string;
  required: boolean;
  defaultValue: string;
  enumValues: string[];
}

export interface LlmConfig {
  autoCloseOnComplete: boolean;
  updateStatusOnStart: boolean;
  allowDirectEdits: boolean;
}

export interface UpdateConfigRequest {
  projectPath: string;
  config: Partial<Config>;
}

export interface UpdateConfigResponse {
  success: boolean;
  error: string;
  config?: Config;
}

export interface GetManifestRequest {
  projectPath: string;
}

export interface Manifest {
  schemaVersion: number;
  centyVersion: string;
  createdAt: string;
  updatedAt: string;
}

export interface FileInfo {
  path: string;
  fileType: string;
  hash: string;
  contentPreview: string;
}

export interface GetDaemonInfoRequest {}

export interface DaemonInfo {
  version: string;
  availableVersions: string[];
}

export interface GetProjectVersionRequest {
  projectPath: string;
}

export interface ProjectVersionInfo {
  projectVersion: string;
  daemonVersion: string;
  comparison: string;
  degradedMode: boolean;
}

export interface UpdateVersionRequest {
  projectPath: string;
  targetVersion: string;
}

export interface UpdateVersionResponse {
  success: boolean;
  error: string;
  fromVersion: string;
  toVersion: string;
  migrationsApplied: string[];
}

export interface ShutdownRequest {
  delaySeconds?: number;
}

export interface ShutdownResponse {
  success: boolean;
  message: string;
}

export interface RestartRequest {
  delaySeconds?: number;
}

export interface RestartResponse {
  success: boolean;
  message: string;
}

/**
 * Create a gRPC client for the Centy daemon.
 * Uses plain text (insecure) transport for testing.
 */
export function createGrpcClient(
  address: string = '127.0.0.1:50051'
): CentyClient {
  const CentyDaemon = protoDescriptor.centy.CentyDaemon;

  const client = new CentyDaemon(
    address,
    grpc.credentials.createInsecure()
  ) as CentyClient;

  return client;
}

/**
 * Promisified wrapper for gRPC client methods.
 */
export function promisifyClient(client: CentyClient) {
  const promisify =
    <TReq, TRes>(method: (req: TReq, cb: (err: any, res: TRes) => void) => void) =>
    (request: TReq): Promise<TRes> =>
      new Promise((resolve, reject) => {
        method.call(client, request, (err: any, response: TRes) => {
          if (err) reject(err);
          else resolve(response);
        });
      });

  return {
    // Init
    init: promisify<InitRequest, InitResponse>(client.init),
    getReconciliationPlan: promisify<GetReconciliationPlanRequest, ReconciliationPlan>(
      client.getReconciliationPlan
    ),
    executeReconciliation: promisify<ExecuteReconciliationRequest, InitResponse>(
      client.executeReconciliation
    ),
    isInitialized: promisify<IsInitializedRequest, IsInitializedResponse>(
      client.isInitialized
    ),

    // Issues
    createIssue: promisify<CreateIssueRequest, CreateIssueResponse>(client.createIssue),
    getIssue: promisify<GetIssueRequest, Issue>(client.getIssue),
    getIssueByDisplayNumber: promisify<GetIssueByDisplayNumberRequest, Issue>(
      client.getIssueByDisplayNumber
    ),
    listIssues: promisify<ListIssuesRequest, ListIssuesResponse>(client.listIssues),
    updateIssue: promisify<UpdateIssueRequest, UpdateIssueResponse>(client.updateIssue),
    deleteIssue: promisify<DeleteIssueRequest, DeleteIssueResponse>(client.deleteIssue),
    getNextIssueNumber: promisify<GetNextIssueNumberRequest, GetNextIssueNumberResponse>(
      client.getNextIssueNumber
    ),

    // Docs
    createDoc: promisify<CreateDocRequest, CreateDocResponse>(client.createDoc),
    getDoc: promisify<GetDocRequest, Doc>(client.getDoc),
    listDocs: promisify<ListDocsRequest, ListDocsResponse>(client.listDocs),
    updateDoc: promisify<UpdateDocRequest, UpdateDocResponse>(client.updateDoc),
    deleteDoc: promisify<DeleteDocRequest, DeleteDocResponse>(client.deleteDoc),

    // Assets
    addAsset: promisify<AddAssetRequest, AddAssetResponse>(client.addAsset),
    listAssets: promisify<ListAssetsRequest, ListAssetsResponse>(client.listAssets),
    getAsset: promisify<GetAssetRequest, GetAssetResponse>(client.getAsset),
    deleteAsset: promisify<DeleteAssetRequest, DeleteAssetResponse>(client.deleteAsset),
    listSharedAssets: promisify<ListSharedAssetsRequest, ListAssetsResponse>(
      client.listSharedAssets
    ),

    // Projects
    listProjects: promisify<ListProjectsRequest, ListProjectsResponse>(client.listProjects),
    registerProject: promisify<RegisterProjectRequest, RegisterProjectResponse>(
      client.registerProject
    ),
    untrackProject: promisify<UntrackProjectRequest, UntrackProjectResponse>(
      client.untrackProject
    ),
    getProjectInfo: promisify<GetProjectInfoRequest, GetProjectInfoResponse>(
      client.getProjectInfo
    ),

    // Config
    getConfig: promisify<GetConfigRequest, Config>(client.getConfig),
    updateConfig: promisify<UpdateConfigRequest, UpdateConfigResponse>(client.updateConfig),
    getManifest: promisify<GetManifestRequest, Manifest>(client.getManifest),

    // Daemon control
    getDaemonInfo: promisify<GetDaemonInfoRequest, DaemonInfo>(client.getDaemonInfo),
    getProjectVersion: promisify<GetProjectVersionRequest, ProjectVersionInfo>(
      client.getProjectVersion
    ),
    updateVersion: promisify<UpdateVersionRequest, UpdateVersionResponse>(
      client.updateVersion
    ),
    shutdown: promisify<ShutdownRequest, ShutdownResponse>(client.shutdown),
    restart: promisify<RestartRequest, RestartResponse>(client.restart),

    // Close connection
    close: () => client.close(),
  };
}

export type PromisifiedCentyClient = ReturnType<typeof promisifyClient>;
