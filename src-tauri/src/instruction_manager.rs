use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

use crate::utils::validation::{validate_instruction_filename, is_allowed_instruction_file};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionFileInfo {
    pub id: String,              // relative path (used as unique ID)
    pub name: String,            // filename for display
    pub path: String,            // full absolute path
    pub relative_path: String,   // path relative to .grove-instructions/
    pub file_type: String,       // "txt" or "md"
    pub size: u64,               // bytes
    pub modified: String,        // ISO 8601 timestamp
}

/// Scan .grove-instructions/ directory for .txt and .md files
/// Also checks for bundled resources in release builds
pub fn list_instruction_files(working_dir: &str) -> Result<Vec<InstructionFileInfo>, String> {
    let instructions_dir = Path::new(working_dir).join(".grove-instructions");

    // If directory doesn't exist, try to initialize from bundled resources
    if !instructions_dir.exists() {
        // Try to find bundled resources and copy them
        if let Err(e) = initialize_instruction_files(working_dir) {
            eprintln!("Warning: Could not initialize instruction files from bundle: {}", e);
        }
    }

    // Try again after initialization
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

/// Initialize .grove-instructions directory with bundled default files
/// In development, this copies from the project's .grove-instructions
/// In release, this uses bundled resources
fn initialize_instruction_files(working_dir: &str) -> Result<(), String> {
    let dest_dir = Path::new(working_dir).join(".grove-instructions");

    // Create directory
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create .grove-instructions directory: {}", e))?;

    // Try to find source files
    // In development: use relative path from executable
    // In release: use bundled resources path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_dir = exe_path.parent()
        .ok_or("Failed to get executable directory")?;

    // Try multiple possible locations for bundled resources
    let possible_sources = vec![
        exe_dir.join(".grove-instructions"),                    // Adjacent to exe (some bundles)
        exe_dir.join("../Resources/.grove-instructions"),       // macOS app bundle
        exe_dir.join("resources/.grove-instructions"),          // Windows/Linux bundle
        exe_dir.join("../../.grove-instructions"),              // Development (target/debug or target/release)
        exe_dir.join("../../../.grove-instructions"),           // Development alternate structure
    ];

    for source_dir in possible_sources {
        if source_dir.exists() && source_dir.is_dir() {
            // Copy all instruction files
            copy_directory_contents(&source_dir, &dest_dir)?;
            return Ok(());
        }
    }

    // If no bundled resources found, create empty directory (not an error)
    Ok(())
}

/// Copy contents of one directory to another (recursively, only .md and .txt files)
fn copy_directory_contents(src: &Path, dest: &Path) -> Result<(), String> {
    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read source directory: {}", e))?;

    let mut copied_count = 0;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively copy subdirectories
            let dir_name = path.file_name()
                .ok_or("Invalid directory name")?;
            let dest_subdir = dest.join(dir_name);

            fs::create_dir_all(&dest_subdir)
                .map_err(|e| format!("Failed to create subdirectory: {}", e))?;

            copy_directory_contents(&path, &dest_subdir)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "txt" || ext == "md" {
                    let filename = path.file_name()
                        .ok_or("Invalid filename")?;
                    let dest_path = dest.join(filename);

                    // Only copy if destination doesn't exist (don't overwrite user files)
                    if !dest_path.exists() {
                        fs::copy(&path, &dest_path)
                            .map_err(|e| format!("Failed to copy file: {}", e))?;
                        copied_count += 1;
                    }
                }
            }
        }
    }

    if copied_count > 0 {
        eprintln!("Initialized .grove-instructions with {} bundled files", copied_count);
    }

    Ok(())
}

fn scan_directory(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<InstructionFileInfo>
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

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

                let modified = metadata.modified()
                    .map_err(|e| format!("Failed to get modified time: {}", e))?;

                // Convert to ISO 8601 string
                let modified_iso = {
                    use std::time::UNIX_EPOCH;
                    let duration = modified.duration_since(UNIX_EPOCH)
                        .map_err(|e| format!("Invalid modified time: {}", e))?;
                    let secs = duration.as_secs();

                    // Simple ISO 8601 formatting (UTC)
                    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
                        .ok_or_else(|| "Failed to convert timestamp".to_string())?;
                    datetime.to_rfc3339()
                };

                // Get relative path from .grove-instructions/
                let relative_path = path.strip_prefix(base_dir)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?
                    .to_string_lossy()
                    .to_string();

                let filename = path.file_name()
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

/// Save instruction file content
pub fn save_instruction_file(working_dir: &str, filename: &str, content: &str) -> Result<(), String> {
    let instructions_dir = Path::new(working_dir).join(".grove-instructions");

    if !instructions_dir.exists() {
        fs::create_dir_all(&instructions_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
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
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if metadata.len() > 100_000 {
        return Err("File too large (max 100KB for preview)".to_string());
    }

    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file content: {}", e))
}

/// Copy selected instruction files to .claude/ directory
/// Returns list of copied file paths for cleanup tracking
pub fn copy_instruction_files(
    working_dir: &str,
    selected_files: &[String]
) -> Result<Vec<String>, String> {
    let instructions_dir = Path::new(working_dir).join(".grove-instructions");
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
        // Example: my-instruction.md -> grove_instruction_my-instruction.md
        let filename = source_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", relative_path))?;

        let prefixed_name = format!("grove_instruction_{}", filename);
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
    copied_files: &[String]
) -> Result<(), String> {
    for file_path in copied_files {
        let path = Path::new(file_path);

        // Safety check: only delete files with grove_instruction_ prefix
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with("grove_instruction_") {
                if path.exists() {
                    fs::remove_file(path)
                        .map_err(|e| format!("Failed to delete {}: {}", file_path, e))?;
                }
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

    #[test]
    fn test_list_instruction_files_empty() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_str().unwrap();

        let result = list_instruction_files(working_dir).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_list_instruction_files_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let instructions_dir = temp_dir.path().join(".grove-instructions");
        fs::create_dir_all(&instructions_dir).unwrap();

        // Create test files
        fs::write(instructions_dir.join("test1.txt"), "content1").unwrap();
        fs::write(instructions_dir.join("test2.md"), "content2").unwrap();
        fs::write(instructions_dir.join("ignore.json"), "{}").unwrap();

        let result = list_instruction_files(temp_dir.path().to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|f| f.name == "test1.txt"));
        assert!(result.iter().any(|f| f.name == "test2.md"));
    }

    #[test]
    fn test_copy_and_cleanup_instruction_files() {
        let temp_dir = TempDir::new().unwrap();
        let working_dir = temp_dir.path().to_str().unwrap();
        let instructions_dir = temp_dir.path().join(".grove-instructions");
        let claude_dir = temp_dir.path().join(".claude");

        fs::create_dir_all(&instructions_dir).unwrap();
        fs::write(instructions_dir.join("test.txt"), "test content").unwrap();

        // Copy files
        let copied = copy_instruction_files(
            working_dir,
            &["test.txt".to_string()]
        ).unwrap();

        assert_eq!(copied.len(), 1);
        assert!(claude_dir.join("grove_instruction_test.txt").exists());

        // Cleanup
        cleanup_instruction_files(working_dir, &copied).unwrap();
        assert!(!claude_dir.join("grove_instruction_test.txt").exists());
    }

    #[test]
    fn test_get_instruction_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let content = get_instruction_file_content(file_path.to_str().unwrap()).unwrap();
        assert_eq!(content, "test content");
    }
}
