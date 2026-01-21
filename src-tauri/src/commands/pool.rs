// Pool-related Tauri commands

use crate::pool_manager::{PoolConfig, PoolStats};
use crate::AppState;

#[tauri::command]
pub async fn get_pool_stats(state: tauri::State<'_, AppState>) -> Result<PoolStats, String> {
    if let Some(pool_arc) = &state.agent_pool {
        let pool = pool_arc.lock().await;
        Ok(pool.get_stats().await)
    } else {
        Err("Pool not initialized".to_string())
    }
}

#[tauri::command]
pub async fn configure_pool(
    _config: PoolConfig,
    _state: tauri::State<'_, AppState>,
    _app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // This would require mutable access to AppState which isn't supported
    // For now, pool configuration is set at startup only
    Err("Pool reconfiguration not yet supported".to_string())
}

#[tauri::command]
pub async fn request_pool_agent(state: tauri::State<'_, AppState>) -> Result<String, String> {
    if let Some(pool_arc) = &state.agent_pool {
        let mut pool = pool_arc.lock().await;
        pool.acquire_agent().await.map_err(|e| e.to_string())
    } else {
        Err("Pool not initialized".to_string())
    }
}

#[tauri::command]
pub async fn release_pool_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(pool_arc) = &state.agent_pool {
        let mut pool = pool_arc.lock().await;
        pool.release_agent(agent_id).await;
        Ok(())
    } else {
        Err("Pool not initialized".to_string())
    }
}
