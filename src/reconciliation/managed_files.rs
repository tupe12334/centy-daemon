use crate::manifest::ManagedFileType;
use std::collections::HashMap;

/// Template for a managed file
#[derive(Debug, Clone)]
pub struct ManagedFileTemplate {
    pub file_type: ManagedFileType,
    pub content: Option<String>,
}

/// Default README content
const README_CONTENT: &str = r#"# Centy Project

This folder is managed by [Centy](https://github.com/tupe12334/centy-cli).

## Structure

- `issues/` - Project issues
- `docs/` - Project documentation
- `assets/` - Shared assets

## Getting Started

Create a new issue:

```bash
centy create issue
```

View all issues in the `issues/` folder.
"#;

/// Get the list of managed files with their templates
pub fn get_managed_files() -> HashMap<String, ManagedFileTemplate> {
    let mut files = HashMap::new();

    files.insert(
        "issues/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "docs/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "assets/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(README_CONTENT.to_string()),
        },
    );

    files
}
