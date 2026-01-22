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

        // Data Shipping Tools - Chain agent work together
        tools.push(Tool {
            name: "ShipDataToAgent".to_string(),
            description: "Send data from one agent's output to another agent as context. Use this to chain agent work together - e.g., Agent A analyzes code, then you ship that analysis to Agent B to write tests based on it. The source agent's output becomes context for the target agent's next task.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source_agent_id": {
                        "type": "string",
                        "description": "The agent ID to get data from (the agent whose output you want to share)"
                    },
                    "target_agent_id": {
                        "type": "string",
                        "description": "The agent ID to send data to (the agent that will receive the context)"
                    },
                    "prompt_with_context": {
                        "type": "string",
                        "description": "The new task/prompt to send to the target agent. The source agent's output will be prepended as context."
                    },
                    "data_selector": {
                        "type": "string",
                        "enum": ["last_output", "all_outputs", "final_result"],
                        "description": "What data to ship from source agent. 'last_output' = most recent text output, 'all_outputs' = all text outputs, 'final_result' = the final result summary. Default: last_output"
                    }
                },
                "required": ["source_agent_id", "target_agent_id", "prompt_with_context"]
            }),
        });

        tools.push(Tool {
            name: "CreateChainedAgent".to_string(),
            description: "Create a new agent that automatically receives context from an existing agent's output. This is a convenience tool that combines creating an agent and shipping data in one step. The new agent starts working immediately with the previous agent's results as context.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "source_agent_id": {
                        "type": "string",
                        "description": "The agent ID to get context from (the agent whose output the new agent should know about)"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "The working directory for the new agent"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "The task for the new agent. The source agent's output will be automatically included as context."
                    }
                },
                "required": ["source_agent_id", "working_dir", "prompt"]
            }),
        });

        // Quick Actions - Common operations
        tools.push(Tool {
            name: "QuickAction".to_string(),
            description: "Execute common quick actions. Available actions: 'status' (list all agents and their status), 'stop_all' (stop all running agents), 'queue' (show the result queue status), 'clear_completed' (clear completed agents from display).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["status", "stop_all", "queue", "clear_completed"],
                        "description": "The quick action to execute"
                    }
                },
                "required": ["action"]
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
