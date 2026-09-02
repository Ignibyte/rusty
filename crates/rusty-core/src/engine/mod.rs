//! Core engine module: Claude CLI subprocess execution, task management, and storage.

pub mod agent_manager;
pub mod agent_prompt_builder;
pub mod conversation_archive;
pub mod db;
pub mod mcp_config;
pub mod memory_manager;
pub mod output_parser;
pub mod process_manager;
pub mod secrets_manager;
pub mod settings_manager;
pub mod task_manager;
pub mod task_queue;
pub mod tool_registry;
pub mod tools;
pub mod user_tasks;
pub mod workflow;
