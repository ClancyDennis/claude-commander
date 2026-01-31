//! Prompt Generator for Commander Personality
//!
//! Uses an LLM to thread personality settings naturally throughout the base
//! system prompt, producing a cohesive document where preferences are woven
//! into relevant sections.

use crate::ai_client::{AIClient, ContentBlock, Message};
use crate::commands::config_loader::get_config_dir;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Cached prompt data persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCache {
    /// The generated/personalized system prompt
    pub prompt: String,
    /// Hash of the personality settings used to generate it
    pub settings_hash: u64,
    /// The personality settings (for debugging/reference)
    pub personality: CommanderPersonality,
}

/// Commander personality settings matching the frontend TypeScript interface
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct CommanderPersonality {
    /// Strictness level (1-10): 1=lenient, 10=demanding
    pub strictness: u8,
    /// Communication style: concise, balanced, verbose
    pub communication_style: String,
    /// Tone: professional, friendly, direct
    pub tone: String,

    /// Preferred programming languages
    pub preferred_languages: Vec<String>,
    /// Preferred frameworks
    pub preferred_frameworks: Vec<String>,
    /// Patterns to favor (free text)
    pub patterns_to_favor: String,
    /// Patterns to avoid (free text)
    pub patterns_to_avoid: String,

    /// Quality focus areas (e.g., "type-safety", "error-handling")
    pub focus_areas: Vec<String>,

    /// Agent autonomy level (1-10): 1=hand-holding, 10=full autonomy
    pub autonomy_level: u8,

    /// Voice attention mode enabled
    pub attention_enabled: bool,
    /// Listen timeout in seconds
    pub listen_timeout: u8,
    /// OpenAI voice: alloy, ash, ballad, coral, echo, sage, shimmer, verse
    pub openai_voice: String,
    /// Voice persona: professional, casual, brief
    pub voice_persona: String,

    /// Custom instructions (free text)
    pub custom_instructions: String,
}

impl Default for CommanderPersonality {
    fn default() -> Self {
        Self {
            strictness: 5,
            communication_style: "balanced".to_string(),
            tone: "professional".to_string(),
            preferred_languages: Vec::new(),
            preferred_frameworks: Vec::new(),
            patterns_to_favor: String::new(),
            patterns_to_avoid: String::new(),
            focus_areas: Vec::new(),
            autonomy_level: 5,
            attention_enabled: false,
            listen_timeout: 0,
            openai_voice: String::new(),
            voice_persona: String::new(),
            custom_instructions: String::new(),
        }
    }
}

impl CommanderPersonality {
    /// Calculate a hash of the settings for cache invalidation
    pub fn settings_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if this is the default/empty personality (no customization)
    pub fn is_default(&self) -> bool {
        self.strictness == 5
            && self.communication_style == "balanced"
            && self.tone == "professional"
            && self.preferred_languages.is_empty()
            && self.preferred_frameworks.is_empty()
            && self.patterns_to_favor.is_empty()
            && self.patterns_to_avoid.is_empty()
            && self.focus_areas.is_empty()
            && self.autonomy_level == 5
            && self.custom_instructions.is_empty()
    }
}

// =============================================================================
// Prompt Cache Persistence
// =============================================================================

const PROMPT_CACHE_FILENAME: &str = "commander_prompt_cache.json";

/// Get the path to the prompt cache file
fn get_cache_path() -> Option<PathBuf> {
    get_config_dir().ok().map(|d| d.join(PROMPT_CACHE_FILENAME))
}

/// Load the cached prompt from disk if it exists
pub fn load_cached_prompt() -> Option<PromptCache> {
    let cache_path = get_cache_path()?;

    if !cache_path.exists() {
        eprintln!("[PromptCache] No cached prompt found at {:?}", cache_path);
        return None;
    }

    match std::fs::read_to_string(&cache_path) {
        Ok(content) => match serde_json::from_str::<PromptCache>(&content) {
            Ok(cache) => {
                eprintln!(
                    "[PromptCache] Loaded cached prompt ({} chars, hash: {})",
                    cache.prompt.len(),
                    cache.settings_hash
                );
                Some(cache)
            }
            Err(e) => {
                eprintln!("[PromptCache] Failed to parse cache file: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("[PromptCache] Failed to read cache file: {}", e);
            None
        }
    }
}

/// Save the prompt cache to disk
pub fn save_cached_prompt(cache: &PromptCache) -> Result<(), String> {
    let cache_path = get_cache_path()
        .ok_or_else(|| "Could not determine config directory for prompt cache".to_string())?;

    // Ensure parent directory exists
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize prompt cache: {}", e))?;

    std::fs::write(&cache_path, json)
        .map_err(|e| format!("Failed to write prompt cache: {}", e))?;

    eprintln!(
        "[PromptCache] Saved prompt cache to {:?} ({} chars)",
        cache_path,
        cache.prompt.len()
    );

    Ok(())
}

/// Clear the cached prompt from disk
pub fn clear_cached_prompt() -> Result<(), String> {
    if let Some(cache_path) = get_cache_path() {
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)
                .map_err(|e| format!("Failed to remove prompt cache: {}", e))?;
            eprintln!("[PromptCache] Cleared cached prompt");
        }
    }
    Ok(())
}

/// Template for the prompt merge instruction
const MERGE_PROMPT_TEMPLATE: &str = r#"You are a prompt engineer. Your task is to take a base system prompt and thread user personality preferences naturally throughout it.

## Base Prompt
```
{base_prompt}
```

## User Personality Settings
{settings_json}

## Instructions
1. DO NOT add new section headers (##) - thread preferences INTO existing sections
2. Modify phrasing in existing sections to reflect the personality:
   - Strictness/tone → adjust language in "How to Work" and "Important Guidelines"
   - Tech preferences → weave into "Important Guidelines" naturally
   - Autonomy → modify agent management advice in "How to Work"
3. Keep the EXACT same markdown structure (same ## headers, same bullet structure)
4. Preserve ALL existing capabilities and identity - only adjust tone/emphasis
5. If strictness is high (7-10): emphasize thorough review, quality standards
6. If strictness is low (1-3): emphasize speed, pragmatic solutions
7. If autonomy is high (7-10): less micromanagement advice
8. If autonomy is low (1-3): more guidance on checking agent work
9. Custom instructions should be incorporated where they make sense contextually
10. Output ONLY the revised prompt text, no commentary or explanation

Output the complete revised system prompt now:"#;

/// Generate a personalized system prompt by threading settings through the base
///
/// Uses the light model (via LIGHT_TASK_MODEL env var) for fast/cheap generation.
pub async fn generate_personalized_prompt(
    base_prompt: &str,
    personality: &CommanderPersonality,
) -> Result<String, String> {
    // If using default settings, just return the base prompt unchanged
    if personality.is_default() {
        return Ok(base_prompt.to_string());
    }

    // Create light model client for this task
    let client =
        AIClient::light_from_env().map_err(|e| format!("Failed to create light client: {}", e))?;

    let settings_json = serde_json::to_string_pretty(personality)
        .map_err(|e| format!("Failed to serialize personality: {}", e))?;

    let prompt = MERGE_PROMPT_TEMPLATE
        .replace("{base_prompt}", base_prompt)
        .replace("{settings_json}", &settings_json);

    let response = client
        .send_message(vec![Message {
            role: "user".to_string(),
            content: prompt,
        }])
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    // Extract text from response
    let text = response
        .content
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if text.is_empty() {
        return Err("LLM returned empty response".to_string());
    }

    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_personality() {
        let personality = CommanderPersonality::default();
        assert!(personality.is_default());
    }

    #[test]
    fn test_customized_personality_not_default() {
        let personality = CommanderPersonality {
            strictness: 8,
            ..Default::default()
        };
        assert!(!personality.is_default());
    }

    #[test]
    fn test_settings_hash_differs() {
        let p1 = CommanderPersonality::default();
        let p2 = CommanderPersonality {
            strictness: 8,
            ..Default::default()
        };

        assert_ne!(p1.settings_hash(), p2.settings_hash());
    }
}
