use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TODO: Actually implement this

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotTaskInformation {
    pub current_cycle: u32,
    pub tasks: HashMap<String, rmpv::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub components: HashMap<String, rmpv::Value>,
    pub task_info: SnapshotTaskInformation,
}
