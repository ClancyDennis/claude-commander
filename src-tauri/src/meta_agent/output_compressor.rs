// Output compression for tool results
//
// This module handles truncation and compression of large tool outputs
// to prevent context overflow while preserving key information.

use serde_json::Value;

/// Find the largest byte index <= `index` that is a valid UTF-8 char boundary.
/// This prevents panics when slicing strings with multi-byte characters.
fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    // Walk backwards from index to find a char boundary
    let mut i = index;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Find the smallest byte index >= `index` that is a valid UTF-8 char boundary.
fn ceil_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut i = index;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Compresses tool outputs that exceed the configured threshold
pub struct OutputCompressor {
    /// Maximum characters for tool output before truncation
    max_chars: usize,
}

impl OutputCompressor {
    /// Create a new output compressor with the given character limit
    pub fn new(max_chars: usize) -> Self {
        Self { max_chars }
    }

    /// Compress a tool output if it exceeds the threshold
    pub fn compress(&self, output: &Value) -> Value {
        let serialized = serde_json::to_string(output).unwrap_or_default();

        if serialized.len() <= self.max_chars {
            return output.clone();
        }

        // For objects, try to preserve key fields and truncate others
        if let Some(obj) = output.as_object() {
            return self.compress_object(obj);
        }

        // For arrays, limit the number of elements
        if let Some(arr) = output.as_array() {
            return self.compress_array(arr);
        }

        // For strings, truncate with ellipsis
        if let Some(s) = output.as_str() {
            return self.compress_string(s);
        }

        // Fallback: serialize and truncate
        self.truncate_serialized(&serialized)
    }

    /// Compress an object by preserving key fields and truncating large values
    fn compress_object(&self, obj: &serde_json::Map<String, Value>) -> Value {
        let mut result = serde_json::Map::new();

        // Key fields to always preserve (if present)
        let priority_keys = [
            "success",
            "error",
            "agent_id",
            "status",
            "message",
            "type",
            "id",
            "name",
            "result",
            "iteration_info",
        ];

        // First pass: add priority keys
        for key in &priority_keys {
            if let Some(value) = obj.get(*key) {
                let compressed = self.compress_value(value, self.max_chars / 4);
                result.insert(key.to_string(), compressed);
            }
        }

        // Calculate remaining budget
        let used = serde_json::to_string(&result).unwrap_or_default().len();
        let remaining = self.max_chars.saturating_sub(used);

        // Second pass: add other keys if space permits
        let other_keys: Vec<_> = obj
            .keys()
            .filter(|k| !priority_keys.contains(&k.as_str()))
            .collect();

        if !other_keys.is_empty() && remaining > 100 {
            let budget_per_key = remaining / other_keys.len();

            for key in other_keys {
                if let Some(value) = obj.get(key) {
                    let compressed = self.compress_value(value, budget_per_key);
                    result.insert(key.clone(), compressed);
                }
            }
        }

        // If still too large, add truncation notice
        let final_serialized = serde_json::to_string(&result).unwrap_or_default();
        if final_serialized.len() > self.max_chars {
            result.insert("_truncated".to_string(), Value::Bool(true));
            result.insert(
                "_original_size".to_string(),
                Value::Number(serde_json::to_string(obj).unwrap_or_default().len().into()),
            );
        }

        Value::Object(result)
    }

    /// Compress an array by limiting elements
    fn compress_array(&self, arr: &[Value]) -> Value {
        let serialized = serde_json::to_string(arr).unwrap_or_default();

        if serialized.len() <= self.max_chars {
            return Value::Array(arr.to_vec());
        }

        // Calculate how many elements we can keep
        let avg_element_size = serialized.len() / arr.len().max(1);
        let max_elements = (self.max_chars / avg_element_size.max(1)).max(2);

        if arr.len() <= max_elements {
            // Compress individual elements
            let compressed: Vec<Value> = arr
                .iter()
                .map(|v| self.compress_value(v, self.max_chars / arr.len()))
                .collect();
            return Value::Array(compressed);
        }

        // Keep first and last elements, show count in middle
        let half = max_elements / 2;
        let mut result: Vec<Value> = arr.iter().take(half).cloned().collect();

        result.push(Value::String(format!(
            "... {} more items omitted ...",
            arr.len() - max_elements
        )));

        result.extend(arr.iter().skip(arr.len() - half).cloned());

        Value::Array(result)
    }

    /// Compress a string by truncating with ellipsis
    fn compress_string(&self, s: &str) -> Value {
        if s.len() <= self.max_chars {
            return Value::String(s.to_string());
        }

        // Keep beginning and end
        let half = (self.max_chars - 50) / 2; // Leave room for ellipsis message
                                              // Use safe char boundaries to avoid panics with multi-byte UTF-8 chars
        let start_end = floor_char_boundary(s, half);
        let end_start = ceil_char_boundary(s, s.len().saturating_sub(half));
        let truncated = format!(
            "{}... [truncated {} chars] ...{}",
            &s[..start_end],
            s.len() - self.max_chars,
            &s[end_start..]
        );

        Value::String(truncated)
    }

    /// Compress a value with a given budget
    fn compress_value(&self, value: &Value, budget: usize) -> Value {
        let serialized = serde_json::to_string(value).unwrap_or_default();

        if serialized.len() <= budget {
            return value.clone();
        }

        match value {
            Value::String(s) => {
                if s.len() <= budget {
                    value.clone()
                } else {
                    // Use safe char boundary to avoid panics with multi-byte UTF-8 chars
                    let safe_end = floor_char_boundary(s, budget.saturating_sub(20));
                    let truncated = format!("{}... [+{} chars]", &s[..safe_end], s.len() - budget);
                    Value::String(truncated)
                }
            }
            Value::Array(arr) => {
                if arr.len() <= 3 {
                    value.clone()
                } else {
                    Value::String(format!("[Array of {} items]", arr.len()))
                }
            }
            Value::Object(obj) => {
                if serialized.len() <= budget * 2 {
                    value.clone()
                } else {
                    Value::String(format!("{{Object with {} keys}}", obj.len()))
                }
            }
            _ => value.clone(),
        }
    }

    /// Truncate serialized JSON as a fallback
    fn truncate_serialized(&self, serialized: &str) -> Value {
        if serialized.len() <= self.max_chars {
            // Try to parse back
            serde_json::from_str(serialized).unwrap_or(Value::String(serialized.to_string()))
        } else {
            // Use safe char boundary to avoid panics with multi-byte UTF-8 chars
            let safe_end = floor_char_boundary(serialized, self.max_chars.saturating_sub(50));
            let truncated = format!(
                "{}... [truncated, original size: {} chars]",
                &serialized[..safe_end],
                serialized.len()
            );
            Value::String(truncated)
        }
    }
}

impl Default for OutputCompressor {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_small_output_unchanged() {
        let compressor = OutputCompressor::new(1000);
        let input = json!({"success": true, "message": "Done"});
        let output = compressor.compress(&input);
        assert_eq!(input, output);
    }

    #[test]
    fn test_large_string_truncated() {
        let compressor = OutputCompressor::new(100);
        let long_string = "a".repeat(500);
        let input = Value::String(long_string);
        let output = compressor.compress(&input);

        if let Value::String(s) = output {
            assert!(s.len() < 200);
            assert!(s.contains("truncated"));
        } else {
            panic!("Expected string output");
        }
    }

    #[test]
    fn test_large_array_compressed() {
        let compressor = OutputCompressor::new(200);
        let input: Value = (0..100).map(|i| json!({"id": i})).collect();
        let output = compressor.compress(&input);

        if let Value::Array(arr) = output {
            assert!(arr.len() < 100);
        } else {
            panic!("Expected array output");
        }
    }

    #[test]
    fn test_priority_keys_preserved() {
        let compressor = OutputCompressor::new(500);
        let input = json!({
            "success": true,
            "error": null,
            "agent_id": "test-123",
            "huge_data": "x".repeat(10000),
            "status": "running"
        });

        let output = compressor.compress(&input);

        if let Value::Object(obj) = output {
            assert!(obj.contains_key("success"));
            assert!(obj.contains_key("agent_id"));
            assert!(obj.contains_key("status"));
        } else {
            panic!("Expected object output");
        }
    }

    #[test]
    fn test_multibyte_utf8_characters_no_panic() {
        // Test with strings containing multi-byte UTF-8 characters
        // '→' is 3 bytes, '日' is 3 bytes, '🔧' is 4 bytes
        let compressor = OutputCompressor::new(50);

        // String with arrows at various positions
        let arrows = "→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→→";
        let input = Value::String(arrows.to_string());
        let output = compressor.compress(&input);
        assert!(matches!(output, Value::String(_)));

        // String with mixed ASCII and multi-byte chars
        let mixed = "Hello 🔧 World → Test 日本語 More text here that needs truncation";
        let input = Value::String(mixed.to_string());
        let output = compressor.compress(&input);
        assert!(matches!(output, Value::String(_)));

        // Test compress_value with budget that falls mid-character
        let test_str = "abc→def→ghi→jkl";
        let compressor_small = OutputCompressor::new(10);
        let output = compressor_small.compress_value(&Value::String(test_str.to_string()), 7);
        assert!(matches!(output, Value::String(_)));
    }
}
