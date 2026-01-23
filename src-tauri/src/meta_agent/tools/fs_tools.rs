// Filesystem tools for MetaAgent

use serde_json::{json, Value};

use crate::meta_agent::helpers::error;

/// List contents of a directory
pub async fn list_directory(input: Value) -> Value {
    let path = input["path"].as_str().unwrap_or("");
    if path.is_empty() {
        return error("path is required");
    }

    // Expand ~ to home directory
    let expanded_path = expand_home_dir(path);

    // Check if path exists
    let path_obj = std::path::Path::new(&expanded_path);
    if !path_obj.exists() {
        return error(format!("Path '{}' does not exist", expanded_path));
    }

    if !path_obj.is_dir() {
        return error(format!("Path '{}' is not a directory", expanded_path));
    }

    // Read directory contents
    match std::fs::read_dir(&expanded_path) {
        Ok(entries) => {
            let mut items = collect_directory_items(entries);

            // Sort: directories first, then alphabetically
            sort_directory_items(&mut items);

            json!({
                "success": true,
                "path": expanded_path,
                "items": items,
                "count": items.len()
            })
        }
        Err(e) => error(format!("Failed to read directory: {}", e)),
    }
}

/// Expand ~ to home directory (cross-platform)
fn expand_home_dir(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            path.replacen('~', home.to_string_lossy().as_ref(), 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

/// Collect directory entries into JSON items
fn collect_directory_items(entries: std::fs::ReadDir) -> Vec<Value> {
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = if entry.path().is_dir() {
            "directory"
        } else {
            "file"
        };
        items.push(json!({
            "name": file_name,
            "type": file_type,
            "path": entry.path().to_string_lossy().to_string()
        }));
    }
    items
}

/// Sort directory items: directories first, then alphabetically
fn sort_directory_items(items: &mut [Value]) {
    items.sort_by(|a, b| {
        let a_type = a["type"].as_str().unwrap_or("");
        let b_type = b["type"].as_str().unwrap_or("");
        let a_name = a["name"].as_str().unwrap_or("");
        let b_name = b["name"].as_str().unwrap_or("");

        match (a_type, b_type) {
            ("directory", "file") => std::cmp::Ordering::Less,
            ("file", "directory") => std::cmp::Ordering::Greater,
            _ => a_name.cmp(b_name),
        }
    });
}
