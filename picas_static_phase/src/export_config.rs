use std::{cell::RefCell, collections::HashMap};

use crate::callback_group::CallbackGroup;
use serde::Serialize;
use serde_yaml;
use std::fs::File;
use std::io::Write;

#[derive(Serialize)]
struct CBGConfig {
    id: String,
    affinity: Vec<usize>,
    policy: String,
    priority: i32,
}

impl CBGConfig {
    fn new(callback_group: &RefCell<CallbackGroup>) -> Self {
        Self {
            id: callback_group.borrow().id.clone(),
            affinity: vec![callback_group.borrow().get_core_id()],
            policy: "SCHED_FIFO".to_string(),
            priority: callback_group.borrow().get_executor_priority(),
        }
    }
}

pub fn export_config(output_dir: &str, callback_groups: &[RefCell<CallbackGroup>]) {
    let mut config = HashMap::new();
    let mut callback_group_configs = Vec::with_capacity(callback_groups.len());
    for callback_group in callback_groups {
        callback_group_configs.push(CBGConfig::new(callback_group));
    }
    config.insert("callback_groups", callback_group_configs);

    let mut config_file =
        File::create(format!("{output_dir}/config.yaml")).expect("Failed to create output file");
    let yaml_string = serde_yaml::to_string(&config).unwrap();
    config_file
        .write_all(yaml_string.as_bytes())
        .expect("Failed to write to output file");
}
