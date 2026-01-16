// Skill generation related Tauri commands

use crate::skill_generator::{self, GeneratedSkill, SkillContent};
use crate::AppState;

#[tauri::command]
pub async fn generate_skill_from_instruction(
    file_path: String,
    working_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<GeneratedSkill, String> {
    let meta_agent = state.meta_agent.lock().await;
    let ai_client = meta_agent.get_ai_client();

    skill_generator::generate_skill_from_instruction(&file_path, &working_dir, ai_client).await
}

#[tauri::command]
pub async fn list_generated_skills(
    working_dir: String,
) -> Result<Vec<GeneratedSkill>, String> {
    skill_generator::list_generated_skills(&working_dir)
}

#[tauri::command]
pub async fn delete_generated_skill(
    skill_name: String,
    working_dir: String,
) -> Result<(), String> {
    skill_generator::delete_generated_skill(&skill_name, &working_dir)
}

#[tauri::command]
pub async fn get_skill_content(
    skill_name: String,
    working_dir: String,
) -> Result<SkillContent, String> {
    skill_generator::get_skill_content(&skill_name, &working_dir)
}
