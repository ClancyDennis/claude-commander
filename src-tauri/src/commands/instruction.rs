// Instruction file management Tauri commands

use crate::instruction_manager::{self, InstructionFileInfo};

#[tauri::command]
pub async fn list_instruction_files(
    working_dir: String,
) -> Result<Vec<InstructionFileInfo>, String> {
    instruction_manager::list_instruction_files(&working_dir)
}

#[tauri::command]
pub async fn get_instruction_file_content(file_path: String) -> Result<String, String> {
    instruction_manager::get_instruction_file_content(&file_path)
}

#[tauri::command]
pub async fn save_instruction_file(
    working_dir: String,
    filename: String,
    content: String,
) -> Result<(), String> {
    instruction_manager::save_instruction_file(&working_dir, &filename, &content)
}

#[tauri::command]
pub async fn open_instructions_directory() -> Result<(), String> {
    let instructions_dir = dirs::home_dir()
        .map(|d| d.join(".instructions"))
        .ok_or("Could not determine home directory")?;

    // Ensure directory exists
    if !instructions_dir.exists() {
        std::fs::create_dir_all(&instructions_dir)
            .map_err(|e| format!("Failed to create instructions directory: {}", e))?;
    }

    // Open the directory using the system's default file manager
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&instructions_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&instructions_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&instructions_dir)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }

    Ok(())
}
