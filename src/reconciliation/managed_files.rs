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
- `templates/` - Custom templates for issues and docs

## Getting Started

Create a new issue:

```bash
centy create issue
```

View all issues in the `issues/` folder.
"#;

/// Templates README content
const TEMPLATES_README_CONTENT: &str = r#"# Templates

This folder contains templates for creating issues and docs using [Handlebars](https://handlebarsjs.com/) syntax.

## Usage

To use a template, specify the `template` parameter when creating an issue or doc:
- Issues: Place templates in `templates/issues/` (e.g., `bug-report.md`)
- Docs: Place templates in `templates/docs/` (e.g., `api.md`)

## Available Placeholders

### Issue Templates
| Placeholder | Description |
|-------------|-------------|
| `{{title}}` | Issue title |
| `{{description}}` | Issue description |
| `{{priority}}` | Priority number (1 = highest) |
| `{{priority_label}}` | Priority label (e.g., "high", "medium", "low") |
| `{{status}}` | Issue status |
| `{{created_at}}` | Creation timestamp |
| `{{custom_fields}}` | Map of custom field key-value pairs |

### Doc Templates
| Placeholder | Description |
|-------------|-------------|
| `{{title}}` | Document title |
| `{{content}}` | Document content |
| `{{slug}}` | URL-friendly slug |
| `{{created_at}}` | Creation timestamp |
| `{{updated_at}}` | Last update timestamp |

## Handlebars Features

Templates support full Handlebars syntax:

### Conditionals
```handlebars
{{#if description}}
## Description
{{description}}
{{/if}}
```

### Loops
```handlebars
{{#each custom_fields}}
- **{{@key}}:** {{this}}
{{/each}}
```

## Example Templates

### Issue Template (`templates/issues/bug-report.md`)
```handlebars
# Bug: {{title}}

**Priority:** {{priority_label}} | **Status:** {{status}}

## Description
{{description}}

{{#if custom_fields}}
## Additional Info
{{#each custom_fields}}
- {{@key}}: {{this}}
{{/each}}
{{/if}}
```

### Doc Template (`templates/docs/api.md`)
```handlebars
---
title: "{{title}}"
slug: "{{slug}}"
---

# API: {{title}}

{{content}}
```
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

    files.insert(
        "templates/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "templates/issues/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "templates/docs/".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::Directory,
            content: None,
        },
    );

    files.insert(
        "templates/README.md".to_string(),
        ManagedFileTemplate {
            file_type: ManagedFileType::File,
            content: Some(TEMPLATES_README_CONTENT.to_string()),
        },
    );

    files
}
