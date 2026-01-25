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

        tools.push(Tool {
            name: "GetAgentTodoList".to_string(),
            description: "Retrieves the current todo/task list from an agent. Agents use TodoWrite to track their progress through tasks. This tool lets you see what tasks an agent has planned, which are completed, which is in progress, and overall progress percentage. Call without agent_id to get todo lists for ALL agents at once.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "Optional: The unique ID of the agent. If omitted, returns todo lists for ALL agents."
                    }
                },
                "required": []
            }),
        });

        tools.push(Tool {
            name: "SearchRunHistory".to_string(),
            description: "Search through historical agent runs stored in the database. Use this to find past work by directory, status, or keyword. Useful for questions like 'what work was done on project X?' or 'find crashed runs that can be resumed'.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "working_dir": {
                        "type": "string",
                        "description": "Filter by directory path (partial match). Example: '/home/user/project' matches any run in that directory or subdirectories."
                    },
                    "status": {
                        "type": "string",
                        "enum": ["running", "completed", "stopped", "crashed", "waiting_input"],
                        "description": "Filter by run status."
                    },
                    "source": {
                        "type": "string",
                        "enum": ["ui", "meta", "pipeline", "pool", "manual"],
                        "description": "Filter by how the agent was created."
                    },
                    "keyword": {
                        "type": "string",
                        "description": "Search for keyword in the initial prompt. Case-insensitive partial match."
                    },
                    "days_back": {
                        "type": "number",
                        "description": "Limit to runs from the last N days. Default: 30."
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results to return. Default: 20."
                    },
                    "resumable_only": {
                        "type": "boolean",
                        "description": "If true, only show crashed runs that can be resumed."
                    }
                },
                "required": []
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

        // Meta-Agent Todo List
        tools.push(Tool {
            name: "UpdateMetaTodoList".to_string(),
            description: "Update the System Commander's own task list. Use this to track orchestration tasks, agent coordination goals, and overall progress. The todo list will be visible to the user in the UI. Each todo item has: content (what needs to be done), status (pending/in_progress/completed), and optionally activeForm (present tense description like 'Creating agent...'). Call this at the start of multi-step operations to show the user what you're planning, and update it as you complete each step.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "todos": {
                        "type": "array",
                        "description": "The complete updated todo list. Each call replaces the previous list.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": {
                                    "type": "string",
                                    "description": "The task description (imperative form, e.g., 'Create worker agent for tests')"
                                },
                                "status": {
                                    "type": "string",
                                    "enum": ["pending", "in_progress", "completed"],
                                    "description": "Current status of this task"
                                },
                                "activeForm": {
                                    "type": "string",
                                    "description": "Optional: Present continuous form shown during execution (e.g., 'Creating worker agent for tests')"
                                }
                            },
                            "required": ["content", "status"]
                        }
                    }
                },
                "required": ["todos"]
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
