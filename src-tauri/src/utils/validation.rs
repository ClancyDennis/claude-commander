use std::path::Path;

/// Allowed file extensions for instruction files
const ALLOWED_EXTENSIONS: &[&str] = &["md", "txt"];

/// Validates an instruction filename
/// - Must end with .md or .txt
/// - Must not contain path separators
pub fn validate_instruction_filename(filename: &str) -> Result<(), String> {
    // Check for path separators
    if filename.contains('/') || filename.contains('\\') {
        return Err("Filename must not contain path separators".to_string());
    }

    // Check extension
    if !has_allowed_extension(filename) {
        return Err("Filename must end with .md or .txt".to_string());
    }

    Ok(())
}

/// Checks if a filename or path has an allowed extension (.md or .txt)
pub fn has_allowed_extension(filename: &str) -> bool {
    let path = Path::new(filename);
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ALLOWED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Checks if a path has an allowed extension for instruction files
pub fn is_allowed_instruction_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ALLOWED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_instruction_filename_valid() {
        assert!(validate_instruction_filename("readme.md").is_ok());
        assert!(validate_instruction_filename("notes.txt").is_ok());
        assert!(validate_instruction_filename("MY_FILE.MD").is_ok());
        assert!(validate_instruction_filename("test.TXT").is_ok());
    }

    #[test]
    fn test_validate_instruction_filename_invalid_extension() {
        assert!(validate_instruction_filename("script.rs").is_err());
        assert!(validate_instruction_filename("image.png").is_err());
        assert!(validate_instruction_filename("noextension").is_err());
    }

    #[test]
    fn test_validate_instruction_filename_path_separators() {
        assert!(validate_instruction_filename("path/to/file.md").is_err());
        assert!(validate_instruction_filename("path\\to\\file.md").is_err());
        assert!(validate_instruction_filename("../file.md").is_err());
    }

    #[test]
    fn test_has_allowed_extension() {
        assert!(has_allowed_extension("file.md"));
        assert!(has_allowed_extension("file.txt"));
        assert!(has_allowed_extension("file.MD"));
        assert!(!has_allowed_extension("file.rs"));
        assert!(!has_allowed_extension("file"));
    }

    #[test]
    fn test_is_allowed_instruction_file() {
        assert!(is_allowed_instruction_file(Path::new("file.md")));
        assert!(is_allowed_instruction_file(Path::new("/path/to/file.txt")));
        assert!(!is_allowed_instruction_file(Path::new("file.rs")));
    }
}
