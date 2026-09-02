//! Tool handler modules for the AI function registry.
//!
//! Each submodule registers a group of related tools (notes, tasks, memory, navigation,
//! agents) with the central `ToolRegistry`.

pub mod agents;
pub mod brain;
pub mod conversations;
pub mod desktop;
pub mod memory;
pub mod navigation;
pub mod notes;
pub mod tasks;
