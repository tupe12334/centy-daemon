# Configurable file format (md/adoc)

## Summary
Let the user configure in their config file for a project if they want the files to be in md or adoc file format. Default will be md.

## Implementation Plan

### Files to Modify
- `src/config/mod.rs` - Add `fileFormat` field to CentyConfig
- `src/template/engine.rs` - Accept extension parameter in load_template()
- `src/issue/create.rs` - Use config extension instead of hardcoded .md
- `src/issue/crud.rs` - Use config extension, add backward compatibility
- `src/docs/crud.rs` - Use config extension throughout (7 locations)
- `src/reconciliation/managed_files.rs` - Configurable README extensions
- `proto/centy.proto` - Add file_format to Config message

### Config Schema
```json
{
  "fileFormat": "md"  // or "adoc"
}
```

### Tasks
1. Add `file_format` field to CentyConfig with default "md"
2. Create helper function `get_file_extension(config)`
3. Update template engine to use configured extension
4. Update issue creation/CRUD to use configured extension
5. Update doc CRUD to use configured extension
6. Update managed files for configurable README extensions
7. Update proto definition
8. Add backward compatibility (check both extensions when reading)
9. Add unit and integration tests
