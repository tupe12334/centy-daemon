# Remove managedFiles from manifest

Remove the `managedFiles` array from `.centy-manifest.json` as the file tracking functionality is not needed. This simplifies the manifest structure.

## Motivation
The tracking functionality is unnecessary for the project goals.

## Files to Modify

### Core Type Definition
- `src/manifest/types.rs` - Remove `managed_files` field from `CentyManifest`, remove `ManagedFile` struct and `ManagedFileType` enum

### Manifest Operations
- `src/manifest/mod.rs` - Remove `add_file_to_manifest()`, `find_managed_file()`, `create_managed_file()` functions and related tests

### Reconciliation Module
- `src/reconciliation/managed_files.rs` - May need significant changes or removal
- `src/reconciliation/plan.rs` - Remove managed_files logic
- `src/reconciliation/execute.rs` - Remove managed_files tracking

### CRUD Modules
- `src/docs/crud.rs` - Remove manifest file tracking
- `src/issue/crud.rs` - Remove manifest file tracking
- `src/issue/assets.rs` - Remove manifest file tracking

### Server/gRPC
- `src/server/mod.rs` - Remove `manifest_to_proto` conversion of managed_files
- `proto/centy.proto` - Remove `ManagedFile` message, `FileType` enum, and field from `Manifest`

### Manifest Files
- `.centy/.centy-manifest.json` - Remove managedFiles array (keep other fields)
