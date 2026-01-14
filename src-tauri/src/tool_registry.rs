use crate::ai_client::Tool;
use serde_json::json;

pub struct ToolRegistry {
    pub tools: Vec<Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools = Vec::new();

        // Agent Management Tools
        tools.push(Tool {
            name: "CreateWorkerAgent".to_string(),
            description: "Creates a new Claude Code worker agent in a specified working directory and optionally sends it an initial task. Use this when the user wants to create an agent and have it do something. IMPORTANT: If the user provides a task or instruction for the agent, you MUST include it in the initial_prompt parameter to start the agent working immediately. Without an initial_prompt, the agent will just wait idle for input. IMPORTANT: Before creating an agent, ask the user what working directory they want to use, or suggest using their home directory (e.g., /home/username/agent-workspace).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "working_dir": {
                        "type": "string",
                        "description": "The absolute path to the working directory where the agent should operate. Must be a valid, existing directory path on the system. If unsure, ask the user or use ~/agent-workspace."
                    },
                    "initial_prompt": {
                        "type": "string",
                        "description": "The first task/prompt to send to the agent immediately after creation. This should contain the user's instruction for what the agent should do. If the user said 'create an agent and make it write tests', this should be 'Write comprehensive tests for this project'."
                    },
                    "navigate": {
                        "type": "boolean",
                        "description": "If true, automatically switch the UI to show this agent after creation. Defaults to false."
                    }
                },
                "required": ["working_dir"]
            }),
        });

        tools.push(Tool {
            name: "SendPromptToWorker".to_string(),
            description: "Sends a prompt/message to an existing worker agent. The agent will process the prompt and begin working on the task.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "The unique ID of the agent to send the prompt to"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "The prompt/instruction to send to the agent"
                    }
                },
                "required": ["agent_id", "prompt"]
            }),
        });

        tools.push(Tool {
            name: "StopWorkerAgent".to_string(),
            description: "Stops/terminates a running worker agent. The agent process will be killed and cannot be resumed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "The unique ID of the agent to stop"
                    }
                },
                "required": ["agent_id"]
            }),
        });

        tools.push(Tool {
            name: "ListWorkerAgents".to_string(),
            description: "Lists all currently running worker agents with their status, working directories, and IDs.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        });

        tools.push(Tool {
            name: "GetAgentOutput".to_string(),
            description: "Retrieves recent output from a worker agent. Useful for checking what an agent has been doing or its current status.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "The unique ID of the agent"
                    },
                    "last_n": {
                        "type": "number",
                        "description": "Optional: number of recent output entries to retrieve (default: 10)"
                    }
                },
                "required": ["agent_id"]
            }),
        });

        // UI Control Tools
        tools.push(Tool {
            name: "NavigateToAgent".to_string(),
            description: "Switches the UI view to show a specific worker agent. The user will see that agent's output and can interact with it.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "The unique ID of the agent to navigate to"
                    }
                },
                "required": ["agent_id"]
            }),
        });

        tools.push(Tool {
            name: "ToggleToolPanel".to_string(),
            description: "Shows or hides the tool activity panel in the UI. The tool panel displays which tools agents are using in real-time.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "show": {
                        "type": "boolean",
                        "description": "true to show the panel, false to hide it"
                    }
                },
                "required": ["show"]
            }),
        });

        tools.push(Tool {
            name: "ShowNotification".to_string(),
            description: "Displays a notification/toast message to the user in the UI. Useful for alerting the user about important events or status changes.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to display to the user"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["info", "success", "error", "warning"],
                        "description": "The type of notification (affects styling and icon)"
                    }
                },
                "required": ["message", "type"]
            }),
        });

        // Filesystem Tools
        tools.push(Tool {
            name: "ListDirectory".to_string(),
            description: "Lists the contents of a directory on the filesystem. Use this to explore available directories and find valid working directory paths for creating agents. Returns a list of files and directories with their types.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The absolute path to the directory to list. Use '~' for the user's home directory, or provide an absolute path like '/home/username' or '/tmp'."
                    }
                },
                "required": ["path"]
            }),
        });

        Self { tools }
    }

    pub fn get_tool_by_name(&self, name: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.name == name)
    }

    pub fn get_all_tools(&self) -> &[Tool] {
        &self.tools
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
