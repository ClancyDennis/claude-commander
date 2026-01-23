use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::validation::{is_allowed_instruction_file, validate_instruction_filename};

/// Get the global instructions directory (~/.instructions/)
fn get_instructions_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    Ok(home.join(".instructions"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionFileInfo {
    pub id: String,            // relative path (used as unique ID)
    pub name: String,          // filename for display
    pub path: String,          // full absolute path
    pub relative_path: String, // path relative to .instructions/
    pub file_type: String,     // "txt" or "md"
    pub size: u64,             // bytes
    pub modified: String,      // ISO 8601 timestamp
}

/// Scan ~/.instructions/ directory for .txt and .md files
/// The working_dir parameter is kept for API compatibility but is no longer used
pub fn list_instruction_files(_working_dir: &str) -> Result<Vec<InstructionFileInfo>, String> {
    let instructions_dir = get_instructions_dir()?;

    // If directory doesn't exist, return empty list
    if !instructions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    // Recursively walk directory
    scan_directory(&instructions_dir, &instructions_dir, &mut files)?;

    // Sort by name for consistent ordering
    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}

fn scan_directory(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<InstructionFileInfo>,
) -> Result<(), String> {
    let entries =
        fs::read_dir(current_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively scan subdirectories
            scan_directory(base_dir, &path, files)?;
        } else if path.is_file() {
            // Check extension using validation utility
            if is_allowed_instruction_file(&path) {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("txt");
                // Get metadata
                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to read file metadata: {}", e))?;

                let modified = metadata
                    .modified()
                    .map_err(|e| format!("Failed to get modified time: {}", e))?;

                // Convert to ISO 8601 string
                let modified_iso = {
                    use std::time::UNIX_EPOCH;
                    let duration = modified
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| format!("Invalid modified time: {}", e))?;
                    let secs = duration.as_secs();

                    // Simple ISO 8601 formatting (UTC)
                    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
                        .ok_or_else(|| "Failed to convert timestamp".to_string())?;
                    datetime.to_rfc3339()
                };

                // Get relative path from .instructions/
                let relative_path = path
                    .strip_prefix(base_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?
                    .to_string_lossy()
                    .to_string();

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                files.push(InstructionFileInfo {
                    id: relative_path.clone(),
                    name: filename,
                    path: path.to_string_lossy().to_string(),
                    relative_path,
                    file_type: ext.to_string(),
                    size: metadata.len(),
                    modified: modified_iso,
                });
            }
        }
    }

    Ok(())
}

/// Save instruction file content to ~/.instructions/
/// The working_dir parameter is kept for API compatibility but is no longer used
pub fn save_instruction_file(
    _working_dir: &str,
    filename: &str,
    content: &str,
) -> Result<(), String> {
    let instructions_dir = get_instructions_dir()?;

    if !instructions_dir.exists() {
        fs::create_dir_all(&instructions_dir)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Validate filename using shared validation utility
    validate_instruction_filename(filename)?;

    let path = instructions_dir.join(filename);
    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

/// Read instruction file content (for preview)
pub fn get_instruction_file_content(file_path: &str) -> Result<String, String> {
    // Validate file exists and has correct extension
    let path = Path::new(file_path);

    if !path.exists() {
        return Err("File not found".to_string());
    }

    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }

    // Check extension using validation utility
    if !is_allowed_instruction_file(path) {
        return Err("Only .txt and .md files are allowed".to_string());
    }

    // Read content (limit to 100KB for safety)
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to read file: {}", e))?;

    if metadata.len() > 100_000 {
        return Err("File too large (max 100KB for preview)".to_string());
    }

    fs::read_to_string(path).map_err(|e| format!("Failed to read file content: {}", e))
}

/// Copy selected instruction files from ~/.instructions/ to working_dir/.claude/ directory
/// Returns list of copied file paths for cleanup tracking
pub fn copy_instruction_files(
    working_dir: &str,
    selected_files: &[String],
) -> Result<Vec<String>, String> {
    let instructions_dir = get_instructions_dir()?;
    let claude_dir = Path::new(working_dir).join(".claude");

    // Ensure .claude directory exists
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir)
            .map_err(|e| format!("Failed to create .claude directory: {}", e))?;
    }

    let mut copied_files = Vec::new();

    for relative_path in selected_files {
        let source_path = instructions_dir.join(relative_path);

        // Validate source exists
        if !source_path.exists() {
            return Err(format!("Instruction file not found: {}", relative_path));
        }

        // Generate safe filename with prefix
        // Example: my-instruction.md -> instruction_my-instruction.md
        let filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", relative_path))?;

        let prefixed_name = format!("instruction_{}", filename);
        let dest_path = claude_dir.join(&prefixed_name);

        // Copy file
        fs::copy(&source_path, &dest_path)
            .map_err(|e| format!("Failed to copy {}: {}", relative_path, e))?;

        copied_files.push(dest_path.to_string_lossy().to_string());
    }

    Ok(copied_files)
}

/// Delete instruction files from .claude/ directory
pub fn cleanup_instruction_files(
    _working_dir: &str,
    copied_files: &[String],
) -> Result<(), String> {
    for file_path in copied_files {
        let path = Path::new(file_path);

        // Safety check: only delete files with instruction_ prefix
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with("instruction_") && path.exists() {
                fs::remove_file(path)
                    .map_err(|e| format!("Failed to delete {}: {}", file_path, e))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Note: list_instruction_files and save_instruction_file now use ~/.instructions/
    // These tests focus on functions that still use working_dir or absolute paths

    #[test]
    fn test_get_instruction_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let content = get_instruction_file_content(file_path.to_str().unwrap()).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_cleanup_instruction_files() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_str().unwrap();
        let claude_dir = temp_dir.path().join(".claude");

        fs::create_dir_all(&claude_dir).unwrap();
        let test_file = claude_dir.join("instruction_test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Cleanup
        cleanup_instruction_files(working_dir, &[test_file.to_string_lossy().to_string()]).unwrap();
        assert!(!test_file.exists());
    }

    #[test]
    fn test_get_instructions_dir() {
        let dir = get_instructions_dir().unwrap();
        assert!(dir.ends_with(".instructions"));
    }
}
