// Output event builder for consistent event creation

use crate::types::{AgentOutputEvent, OutputMetadata};

/// Builder for creating AgentOutputEvent instances
/// Eliminates duplicated output event creation code
pub struct OutputEventBuilder {
    agent_id: String,
    output_type: String,
    content: String,
    parsed_json: Option<serde_json::Value>,
    language: Option<String>,
    line_count: Option<usize>,
    byte_size: Option<usize>,
    is_truncated: bool,
    session_id: Option<String>,
    uuid: Option<String>,
    parent_tool_use_id: Option<String>,
    subtype: Option<String>,
}

impl OutputEventBuilder {
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            output_type: "text".to_string(),
            content: String::new(),
            parsed_json: None,
            language: None,
            line_count: None,
            byte_size: None,
            is_truncated: false,
            session_id: None,
            uuid: None,
            parent_tool_use_id: None,
            subtype: None,
        }
    }

    pub fn output_type(mut self, output_type: impl Into<String>) -> Self {
        self.output_type = output_type.into();
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn parsed_json(mut self, json: Option<serde_json::Value>) -> Self {
        self.parsed_json = json;
        self
    }

    pub fn language(mut self, language: Option<String>) -> Self {
        self.language = language;
        self
    }

    pub fn session_context(
        mut self,
        session_id: Option<String>,
        uuid: Option<String>,
        parent_tool_use_id: Option<String>,
        subtype: Option<String>,
    ) -> Self {
        self.session_id = session_id;
        self.uuid = uuid;
        self.parent_tool_use_id = parent_tool_use_id;
        self.subtype = subtype;
        self
    }

    pub fn with_size_from_content(mut self) -> Self {
        self.byte_size = Some(self.content.len());
        self.line_count = Some(self.content.lines().count());
        self
    }

    pub fn is_truncated(mut self, truncated: bool) -> Self {
        self.is_truncated = truncated;
        self
    }

    pub fn build(self) -> AgentOutputEvent {
        AgentOutputEvent {
            agent_id: self.agent_id,
            output_type: self.output_type,
            content: self.content,
            parsed_json: self.parsed_json,
            metadata: Some(OutputMetadata {
                language: self.language,
                line_count: self.line_count,
                byte_size: self.byte_size,
                is_truncated: self.is_truncated,
            }),
            session_id: self.session_id,
            uuid: self.uuid,
            parent_tool_use_id: self.parent_tool_use_id,
            subtype: self.subtype,
        }
    }
}

/// Extract common fields from a Claude JSON message
pub fn extract_common_fields(
    json: &serde_json::Value,
) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let session_id = json
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let uuid = json
        .get("uuid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let parent_tool_use_id = json
        .get("parent_tool_use_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let subtype = json
        .get("subtype")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    (session_id, uuid, parent_tool_use_id, subtype)
}
