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
