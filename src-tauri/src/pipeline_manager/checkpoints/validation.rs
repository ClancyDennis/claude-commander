use std::process::Command;

use crate::pipeline_manager::types::CheckpointResult;

/// Run automatic validation using shell command
pub fn run_validation(command: &str, working_dir: &str) -> Result<CheckpointResult, String> {
    println!("Running validation command: {}", command);

    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/C", command])
        .current_dir(working_dir)
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    #[cfg(not(windows))]
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(working_dir)
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    let passed = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let result = if passed {
        CheckpointResult::success(format!("Validation passed: {}", command))
    } else {
        CheckpointResult::failure(format!(
            "Validation failed: {}\nStderr: {}",
            command, stderr
        ))
    };

    Ok(result.with_details(serde_json::json!({
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string(),
        "exit_code": output.status.code(),
    })))
}
