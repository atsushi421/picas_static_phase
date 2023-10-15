use crate::callback::{Callback, RegularCallback, TimerCallback};
use crate::callback_group::CallbackGroup;
use crate::chain::Chain;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::rc::Rc;

use crate::graph_extension::{GraphExtension, NodeData};

use petgraph::{graph::Graph, prelude::*};
use yaml_rust::YamlLoader;

fn load_yaml(file_path: &str) -> Vec<yaml_rust::Yaml> {
    if !file_path.ends_with(".yaml") && !file_path.ends_with(".yml") {
        panic!("Invalid file type: {}", file_path);
    }
    let file_content = fs::read_to_string(file_path).unwrap();
    YamlLoader::load_from_str(&file_content).unwrap()
}

fn create_dag_from_yaml(file_path: &str) -> Graph<NodeData, i32> {
    let yaml_doc = &load_yaml(file_path)[0];
    let mut dag = Graph::<NodeData, i32>::new();

    // add nodes to dag
    for node in yaml_doc["nodes"].as_vec().unwrap() {
        let mut id = 0;
        let mut name = "0".to_string();
        let mut callback_group_id = "0".to_string();
        let mut wcet = 0;
        let mut period = None;
        for (key, value) in node.as_hash().unwrap() {
            let key_str = key.as_str().unwrap();
            if key_str == "id" {
                id = value.as_i64().unwrap() as usize;
            } else if key_str == "name" {
                name = value.as_str().unwrap().to_string();
            } else if key_str == "callback_group_id" {
                callback_group_id = value.as_str().unwrap().to_string();
            } else if key_str == "wcet" {
                wcet = value.as_i64().unwrap() as i32;
            } else if key_str == "period" {
                period = Some(value.as_i64().unwrap() as i32);
            } else {
                panic!("Invalid key: {}", key_str);
            }
        }
        dag.add_node(NodeData::new(id, &name, &callback_group_id, wcet, period));
    }

    // add edges to dag
    for link in yaml_doc["links"].as_vec().unwrap() {
        let source = link["source"].as_i64().unwrap() as usize;
        let target = link["target"].as_i64().unwrap() as usize;
        let dummy_communication_time = 0;

        dag.add_edge(
            NodeIndex::new(source),
            NodeIndex::new(target),
            dummy_communication_time,
        );
    }
    dag
}

fn get_weakly_connected_components(dag: &Graph<NodeData, i32>) -> Vec<Graph<NodeData, i32>> {
    let mut visited = HashSet::new();
    let mut all_components = Vec::new();

    for node in dag.node_indices() {
        if visited.contains(&node) {
            continue;
        }

        let mut dfs = Dfs::new(&dag, node);
        let mut component_nodes = Vec::new();

        while let Some(next) = dfs.next(&dag) {
            component_nodes.push(next);
            visited.insert(next);
        }

        // 新しいサブグラフを作成
        let mut subgraph = Graph::<NodeData, i32>::new();
        let mut node_mapping = HashMap::new();

        for &node_idx in &component_nodes {
            let node_data = dag[node_idx].clone();
            let new_node = subgraph.add_node(node_data);
            node_mapping.insert(node_idx, new_node);
        }

        for &node_idx in &component_nodes {
            for edge in dag.edges(node_idx) {
                let source = node_mapping[&edge.source()];
                let target = node_mapping[&edge.target()];
                subgraph.add_edge(source, target, 0);
            }
        }

        all_components.push(subgraph);
    }

    all_components
}

fn split_dag_into_chains(
    dag: &mut Graph<NodeData, i32>,
    current_chain_priority: &mut i32,
) -> (Vec<RefCell<CallbackGroup>>, Vec<Chain>) {
    let dag_period = dag.get_head_period().unwrap();
    let mut chains: Vec<Chain> = Vec::new();
    let mut callback_groups: HashMap<String, Vec<Rc<RefCell<dyn Callback>>>> = HashMap::new();

    while dag.node_count() > 0 {
        split_dag_into_chains_core(
            dag,
            &mut dag.clone(),
            dag_period,
            current_chain_priority,
            &mut chains,
            &mut callback_groups,
        );
    }

    let callback_groups: Vec<RefCell<CallbackGroup>> = callback_groups
        .into_iter()
        .map(|(id, callbacks)| RefCell::new(CallbackGroup::new(&id, callbacks)))
        .collect();
    (callback_groups, chains)
}

fn split_dag_into_chains_core(
    original_dag: &mut Graph<NodeData, i32>,
    dag: &mut Graph<NodeData, i32>,
    dag_period: i32,
    current_chain_priority: &mut i32,
    chains: &mut Vec<Chain>,
    callback_groups: &mut HashMap<String, Vec<Rc<RefCell<dyn Callback>>>>,
) {
    let mut regular_callbacks: Vec<Rc<RefCell<RegularCallback>>> = Vec::new();
    let mut critical_path: Vec<NodeIndex> = dag.get_critical_path();
    let timer_index = critical_path.remove(0);
    let timer_node = dag.node_weight(timer_index).unwrap().clone();
    let timer_callback = Rc::new(RefCell::new(TimerCallback::new(
        timer_node.wcet,
        dag_period,
    )));
    original_dag.remove_node(timer_index);
    callback_groups
        .entry(timer_node.callback_group_id)
        .or_default()
        .push(timer_callback.clone());
    for node_i in critical_path.iter() {
        let node = &dag[*node_i];
        let regular = Rc::new(RefCell::new(RegularCallback::new(node.wcet)));
        regular_callbacks.push(regular.clone());
        callback_groups
            .entry(node.callback_group_id.clone())
            .or_default()
            .push(regular.clone());
    }

    chains.push(Chain::new(
        *current_chain_priority,
        timer_callback.clone(),
        regular_callbacks,
    ));
    *current_chain_priority += 1;
    original_dag.remove_nodes(&critical_path);

    let mut weakly_connected_components = get_weakly_connected_components(original_dag);
    for component in weakly_connected_components.iter_mut() {
        split_dag_into_chains_core(
            original_dag,
            component,
            dag_period,
            current_chain_priority,
            chains,
            callback_groups,
        );
    }
}

pub fn parse_dags(dir_path: &str) -> (Vec<RefCell<CallbackGroup>>, Vec<Chain>) {
    let mut current_chain_priority = 0;

    let mut perception_dag = create_dag_from_yaml(&format!("{dir_path}/perception.yaml"));
    let (mut perception_cbgs, mut perception_chains) =
        split_dag_into_chains(&mut perception_dag, &mut current_chain_priority);

    let mut sensing_localization_dag =
        create_dag_from_yaml(&format!("{dir_path}/sensing_localization.yaml"));
    let (mut sl_cbgs, mut sl_chains) =
        split_dag_into_chains(&mut sensing_localization_dag, &mut current_chain_priority);

    perception_cbgs.append(&mut sl_cbgs);
    perception_chains.append(&mut sl_chains);
    (perception_cbgs, perception_chains)
}
