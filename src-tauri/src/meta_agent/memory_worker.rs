// Memory Worker - Async background processor for memory updates
//
// Provides non-blocking memory updates so the meta-agent can continue
// responding to chat while memory is being processed.
//
// Uses tauri::async_runtime::spawn to run memory tasks as detached async tasks.
// IMPORTANT: Must use tauri::async_runtime::spawn instead of tokio::spawn because
// tokio::spawn can silently fail in Tauri apps (tasks start but async ops don't complete).
// See: https://github.com/tauri-apps/tauri/discussions/11831

use crate::ai_client::{AIClient, ContentBlock, Message};

use super::memory_manager::MemoryManager;

/// Minimum messages required for evaluation (skip if too few)
const MIN_MESSAGES_FOR_EVAL: usize = 3;

/// Background worker for async memory updates
///
/// Uses tauri::async_runtime::spawn to run memory tasks as detached async tasks.
/// Creates a new AIClient for each request to avoid lifetime/clone issues.
#[derive(Clone)]
pub struct MemoryWorker {
    // No state needed - we create a fresh client for each request
    _private: (),
}

impl MemoryWorker {
    /// Create a new memory worker (uses tauri::async_runtime::spawn)
    pub fn start() -> Self {
        eprintln!("[MemoryWorker] Memory worker created (uses tauri::async_runtime::spawn)");
        Self { _private: () }
    }

    /// Queue an update request (spawns detached task)
    pub fn queue_update(&self, instruction: String) {
        eprintln!(
            "[MemoryWorker] Queuing update: {}",
            &instruction[..instruction.len().min(50)]
        );

        // Use tauri::async_runtime::spawn instead of tokio::spawn
        // (tokio::spawn can silently fail in Tauri apps)
        tauri::async_runtime::spawn(async move {
            if let Err(e) = process_update(&instruction).await {
                eprintln!("[MemoryWorker] Update failed: {}", e);
            } else {
                eprintln!("[MemoryWorker] Update completed successfully");
            }
        });
    }

    /// Queue an evaluation request (spawns detached task)
    pub fn queue_evaluation(&self, recent_messages: Vec<Message>) {
        // Skip if too few messages
        if recent_messages.len() < MIN_MESSAGES_FOR_EVAL {
            eprintln!(
                "[MemoryWorker] Skipping evaluation - only {} messages (need {})",
                recent_messages.len(),
                MIN_MESSAGES_FOR_EVAL
            );
            return;
        }

        eprintln!(
            "[MemoryWorker] Queuing evaluation ({} messages)",
            recent_messages.len()
        );

        // Use tauri::async_runtime::spawn instead of tokio::spawn
        // (tokio::spawn can silently fail in Tauri apps)
        tauri::async_runtime::spawn(async move {
            // Create a fresh light client for this request
            let client = match AIClient::light_from_env() {
                Ok(client) => {
                    eprintln!(
                        "[MemoryWorker] Light client created [{}/{}]",
                        client.get_provider_name(),
                        client.get_model_name()
                    );
                    client
                }
                Err(e) => {
                    eprintln!("[MemoryWorker] Failed to create light client: {:?}", e);
                    return;
                }
            };

            if let Err(e) = process_evaluation(recent_messages, &client).await {
                eprintln!("[MemoryWorker] Evaluation failed: {}", e);
            } else {
                eprintln!("[MemoryWorker] Evaluation completed successfully");
            }
        });
    }
}

/// Process an explicit update request
async fn process_update(instruction: &str) -> Result<(), String> {
    eprintln!("[MemoryWorker] Update Step 1: Creating memory manager");
    let manager = MemoryManager::new().ok_or("Failed to create memory manager")?;

    eprintln!("[MemoryWorker] Update Step 2: Calling update_memory");
    let result = manager.update_memory(instruction).await?;

    eprintln!(
        "[MemoryWorker] Update Step 3: Complete - {} tokens used",
        result.token_count
    );

    Ok(())
}

/// Process an evaluation request - decide if memory needs updating
async fn process_evaluation(
    recent_messages: Vec<Message>,
    client: &AIClient,
) -> Result<(), String> {
    eprintln!("[MemoryWorker] Step 1: Creating memory manager");
    let manager = MemoryManager::new().ok_or("Failed to create memory manager")?;

    eprintln!("[MemoryWorker] Step 2: Reading current memory");
    let current_memory = manager.read_memory();

    eprintln!(
        "[MemoryWorker] Step 3: Formatting {} messages",
        recent_messages.len()
    );
    let formatted_messages = format_messages(&recent_messages);

    eprintln!("[MemoryWorker] Step 4: Building evaluation prompt");
    let system_prompt = build_evaluation_prompt(&current_memory);

    let messages = vec![Message {
        role: "user".to_string(),
        content: formatted_messages,
    }];

    eprintln!(
        "[MemoryWorker] Step 5: Calling API [{}/{}]",
        client.get_provider_name(),
        client.get_model_name()
    );

    // Rely on reqwest's built-in timeout (tokio::time::timeout doesn't work well in spawned tasks)
    let response = client
        .send_message_with_system_and_tools(&system_prompt, messages, vec![])
        .await
        .map_err(|e| format!("Evaluation API call failed: {}", e))?;

    eprintln!("[MemoryWorker] Step 5b: API response received");

    eprintln!("[MemoryWorker] Step 6: Extracting text from response");
    let response_text = extract_text(&response.content);
    let response_trimmed = response_text.trim();

    eprintln!(
        "[MemoryWorker] Step 7: Evaluation result ({} chars): {}",
        response_trimmed.len(),
        &response_trimmed[..response_trimmed.len().min(100)]
    );

    // Check if update is needed
    if response_trimmed == "NO_UPDATE" || response_trimmed.is_empty() {
        eprintln!("[MemoryWorker] Step 8: No memory update needed - done");
        return Ok(());
    }

    // The response is the fact to store
    eprintln!(
        "[MemoryWorker] Step 8: Storing fact: {}",
        &response_trimmed[..response_trimmed.len().min(80)]
    );

    let result = manager.update_memory(response_trimmed).await?;

    eprintln!(
        "[MemoryWorker] Step 9: Memory updated - {} tokens used",
        result.token_count
    );

    Ok(())
}

/// Format messages for the evaluation prompt
fn format_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|m| format!("[{}]: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Build the evaluation system prompt
fn build_evaluation_prompt(current_memory: &str) -> String {
    let memory_display = if current_memory.is_empty() {
        "(empty - no memories stored yet)".to_string()
    } else {
        current_memory.to_string()
    };

    format!(
        r#"You analyze conversations to extract facts worth remembering for a System Commander AI.

Current memory:
---
{}
---

The user will provide recent conversation messages. Your job is to determine if there is anything NEW and IMPORTANT that should be stored in memory.

Look for:
- Project details (names, paths, technologies used)
- Important decisions made
- Corrections to previous understanding
- Key facts about the user's environment

If YES - there is something new worth storing:
Output ONLY the fact to store, in a concise format.
Example: "User prefers TypeScript over JavaScript"
Example: "Working on project claude-commander at ~/Documents/Claude Commander"

If NO - nothing new to store:
Output exactly: NO_UPDATE

Important:
- Only output facts NOT already in the current memory
- Be concise - one fact per line if multiple
- Do not include explanations, just the facts or NO_UPDATE"#,
        memory_display
    )
}

/// Extract text content from response blocks
fn extract_text(content: &[ContentBlock]) -> String {
    content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_format_messages() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi there".to_string(),
            },
        ];

        let formatted = format_messages(&messages);
        assert!(formatted.contains("[user]: Hello"));
        assert!(formatted.contains("[assistant]: Hi there"));
    }

    #[test]
    fn test_build_evaluation_prompt_empty_memory() {
        let prompt = build_evaluation_prompt("");
        assert!(prompt.contains("(empty - no memories stored yet)"));
    }

    #[test]
    fn test_build_evaluation_prompt_with_memory() {
        let prompt = build_evaluation_prompt("User prefers Rust");
        assert!(prompt.contains("User prefers Rust"));
        assert!(!prompt.contains("(empty"));
    }

    // =========================================================================
    // Diagnostic tests for memory worker API timeout issue
    // =========================================================================

    /// Test 1: Can we make HTTP requests from main thread?
    #[test]
    fn test_http_request_main_thread() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let client = reqwest::Client::new();
            client
                .get("https://httpbin.org/get")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
        });

        assert!(result.is_ok(), "HTTP request failed: {:?}", result.err());
        assert!(result.unwrap().status().is_success());
    }

    /// Test 2: Can we make HTTP requests from a spawned thread?
    #[test]
    fn test_http_request_spawned_thread() {
        let handle = thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = reqwest::Client::new();
                client
                    .get("https://httpbin.org/get")
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await
            })
        });

        let result = handle.join().unwrap();
        assert!(
            result.is_ok(),
            "HTTP request in thread failed: {:?}",
            result.err()
        );
    }

    /// Test 3: Can we make HTTP requests with multi_thread runtime in spawned thread?
    #[test]
    fn test_http_request_multi_thread_runtime() {
        let handle = thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let client = reqwest::Client::new();
                client
                    .get("https://httpbin.org/get")
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await
            })
        });

        let result = handle.join().unwrap();
        assert!(result.is_ok(), "HTTP request failed: {:?}", result.err());
    }

    /// Test 4: Can we make OpenAI API call from spawned thread?
    #[test]
    #[ignore] // Run with: cargo test test_openai_api_spawned_thread -- --ignored
    fn test_openai_api_spawned_thread() {
        let api_key =
            std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set for this test");

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({
                        "model": "gpt-4o-mini",
                        "messages": [{"role": "user", "content": "Say 'test' only"}],
                        "max_tokens": 5
                    }))
                    .timeout(std::time::Duration::from_secs(30))
                    .send()
                    .await?;

                let status = response.status();
                let body = response.text().await?;
                Ok::<_, reqwest::Error>((status, body))
            })
        });

        let result = handle.join().unwrap();
        match result {
            Ok((status, body)) => {
                eprintln!("Status: {}", status);
                eprintln!("Body: {}", &body[..body.len().min(200)]);
                assert!(status.is_success(), "API returned error: {}", body);
            }
            Err(e) => panic!("Request failed: {:?}", e),
        }
    }

    /// Test 5: Test with AIClient (our abstraction) from spawned thread
    #[test]
    #[ignore] // Run with: cargo test test_aiclient_spawned_thread -- --ignored
    fn test_aiclient_spawned_thread() {
        let handle = thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let client = AIClient::light_from_env()
                    .map_err(|e| format!("Failed to create client: {:?}", e))?;

                eprintln!(
                    "Client created: {}/{}",
                    client.get_provider_name(),
                    client.get_model_name()
                );

                let messages = vec![Message {
                    role: "user".to_string(),
                    content: "Say 'test' only".to_string(),
                }];

                let response = client
                    .send_message(messages)
                    .await
                    .map_err(|e| format!("API call failed: {:?}", e))?;

                eprintln!("Response received: {} tokens", response.usage.output_tokens);
                Ok::<_, String>(response)
            })
        });

        let result = handle.join().unwrap();
        assert!(result.is_ok(), "AIClient test failed: {:?}", result.err());
    }

    /// Test 8: Test process_evaluation directly (bypassing worker thread)
    #[test]
    #[ignore] // Run with: cargo test test_process_evaluation_direct -- --ignored
    fn test_process_evaluation_direct() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let result = rt.block_on(async {
            eprintln!("[Test] Creating AIClient...");
            let client = AIClient::light_from_env()
                .map_err(|e| format!("Failed to create client: {:?}", e))?;

            eprintln!(
                "[Test] Client created: {}/{}",
                client.get_provider_name(),
                client.get_model_name()
            );

            let messages = vec![
                Message {
                    role: "user".to_string(),
                    content: "I prefer using VS Code for development".to_string(),
                },
                Message {
                    role: "assistant".to_string(),
                    content: "VS Code is a great choice!".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Yes, I use it with the Rust Analyzer extension".to_string(),
                },
            ];

            eprintln!("[Test] Calling process_evaluation directly...");
            process_evaluation(messages, &client).await
        });

        match result {
            Ok(()) => eprintln!("[Test] SUCCESS - process_evaluation completed"),
            Err(e) => panic!("[Test] FAILED - process_evaluation error: {}", e),
        }
    }
}
