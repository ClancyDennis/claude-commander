use tauri::Emitter;

/// Trait to abstract event emission, decoupling from tauri::AppHandle
pub trait AppEventEmitter: Send + Sync {
    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String>;
}

// Implement for tauri::AppHandle
impl<R: tauri::Runtime> AppEventEmitter for tauri::AppHandle<R> {
    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String> {
        Emitter::emit(self, event, payload).map_err(|e| e.to_string())
    }
}
