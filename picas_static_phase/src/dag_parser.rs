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
        let mut callback_group_id = "0".to_string();
        let mut name = "0".to_string();
        let mut wcet = 0;
        let mut period = None;

        for (key, value) in node.as_hash().unwrap() {
            match key.as_str().unwrap() {
                "id" => id = value.as_i64().unwrap() as usize,
                "callback_group_id" => callback_group_id = value.as_str().unwrap().to_string(),
                "name" => name = value.as_str().unwrap().to_string(),
                "wcet" => wcet = value.as_i64().unwrap() as i32,
                "period" => period = Some(value.as_i64().unwrap() as i32),
                _ => unreachable!(),
            }
        }
        dag.add_node(NodeData::new(id, &name, &callback_group_id, wcet, period));
    }

    // add edges to dag
    for link in yaml_doc["links"].as_vec().unwrap() {
        let source = link["source"].as_i64().unwrap() as usize;
        let target = link["target"].as_i64().unwrap() as usize;
        dag.add_edge(NodeIndex::new(source), NodeIndex::new(target), 0);
    }
    dag
}

fn get_weakly_connected_graphs(dag: &Graph<NodeData, i32>) -> Vec<Graph<NodeData, i32>> {
    let mut visited = HashSet::new();
    let mut components_nodes: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new(); // start node index -> component index

    for start_node in dag.node_indices() {
        if visited.contains(&start_node) {
            continue;
        }

        let mut dfs = Dfs::new(&dag, start_node);
        let mut merge_target = None;
        while let Some(next) = dfs.next(&dag) {
            if visited.insert(next) {
                components_nodes.entry(start_node).or_default().push(next);
            } else {
                merge_target = Some(next);
            }
        }

        if let Some(merge_target) = merge_target {
            let key_node = components_nodes
                .iter()
                .find(|(_, component_nodes)| component_nodes.contains(&merge_target))
                .map(|(key_node, _)| *key_node)
                .unwrap();

            for node_idx in components_nodes.remove(&key_node).unwrap() {
                components_nodes
                    .entry(start_node)
                    .or_default()
                    .push(node_idx);
            }
        }
    }

    let mut all_components = Vec::new();
    for (_, component_nodes) in components_nodes {
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
            current_chain_priority,
            dag_period,
            &mut dag.clone(),
            &mut chains,
            &mut callback_groups,
        );
    }

    (
        callback_groups
            .into_iter()
            .map(|(id, callbacks)| RefCell::new(CallbackGroup::new(&id, callbacks)))
            .collect(),
        chains,
    )
}

fn split_dag_into_chains_core(
    original_dag: &mut Graph<NodeData, i32>,
    current_chain_priority: &mut i32,
    dag_period: i32,
    dag: &mut Graph<NodeData, i32>,
    chains: &mut Vec<Chain>,
    callback_groups: &mut HashMap<String, Vec<Rc<RefCell<dyn Callback>>>>,
) {
    let mut regular_callbacks: Vec<Rc<RefCell<RegularCallback>>> = Vec::new();
    let critical_path = dag.get_critical_path();

    // Create a timer callback
    let timer_node = critical_path[0];
    let timer_callback = Rc::new(RefCell::new(TimerCallback::new(
        &timer_node.name,
        timer_node.wcet,
        dag_period,
    )));
    callback_groups
        .entry(timer_node.callback_group_id.clone())
        .or_default()
        .push(timer_callback.clone());

    if critical_path.len() > 1 {
        // Create regular callbacks
        for node in critical_path[1..].iter() {
            let regular = Rc::new(RefCell::new(RegularCallback::new(&node.name, node.wcet)));
            regular_callbacks.push(regular.clone());
            callback_groups
                .entry(node.callback_group_id.clone())
                .or_default()
                .push(regular.clone());
        }
    }
    original_dag.remove_nodes(&critical_path);

    // Create a chain
    chains.push(Chain::new(
        *current_chain_priority,
        timer_callback.clone(),
        regular_callbacks,
    ));
    *current_chain_priority += 1;

    let mut subgraphs = get_weakly_connected_graphs(original_dag);
    if let Some(max_subgraph) = subgraphs
        .iter_mut()
        .max_by_key(|subgraph| subgraph.node_count())
    {
        split_dag_into_chains_core(
            original_dag,
            current_chain_priority,
            dag_period,
            max_subgraph,
            chains,
            callback_groups,
        );
    }
}

pub fn parse_dags(dir_path: &str) -> (Vec<RefCell<CallbackGroup>>, Vec<Chain>) {
    let mut current_chain_priority = 0;

    // Perception has a higher priority than sensing and localization because it is positioned later.
    let mut perception_dag = create_dag_from_yaml(&format!("{dir_path}/perception.yaml"));
    let (mut perception_cbgs, mut perception_chains) =
        split_dag_into_chains(&mut perception_dag, &mut current_chain_priority);

    #[cfg(debug_assertions)]
    {
        for (i, chain) in perception_chains.iter().enumerate() {
            export_chain_to_dot(chain, &format!("{dir_path}/perception_chain_{i}.dot"));
        }
    }

    let mut sensing_localization_dag =
        create_dag_from_yaml(&format!("{dir_path}/sensing_localization.yaml"));
    let (mut sl_cbgs, mut sl_chains) =
        split_dag_into_chains(&mut sensing_localization_dag, &mut current_chain_priority);

    #[cfg(debug_assertions)]
    {
        for (i, chain) in sl_chains.iter().enumerate() {
            export_chain_to_dot(
                chain,
                &format!("{dir_path}/sensing_localization_chain_{i}.dot"),
            );
        }
    }

    perception_cbgs.append(&mut sl_cbgs);
    perception_chains.append(&mut sl_chains);
    (perception_cbgs, perception_chains)
}

#[cfg(debug_assertions)]
fn export_chain_to_dot(chain: &Chain, file_path: &str) {
    let mut dag = Graph::<NodeData, i32>::new();
    dag.add_node(NodeData::new(
        0,
        chain.head_timer_callback.borrow().name.as_str(),
        "dummy",
        0,
        Some(chain.head_timer_callback.borrow().period),
    ));
    for regular in &chain.regular_callbacks {
        let node_i = dag.add_node(NodeData::new(
            0,
            regular.borrow().name.as_str(),
            "dummy",
            0,
            None,
        ));
        dag.add_edge(NodeIndex::new(node_i.index() - 1), node_i, 0);
    }

    let output = format!("{:?}", Dot::with_config(&dag, &[Config::EdgeNoLabel]));
    let mut file = File::create(file_path).expect("Failed to create file");
    file.write_all(output.as_bytes())
        .expect("Failed to write to file");
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DAGCreator {
        dag: Graph<NodeData, i32>,
        current_node_id: usize,
    }

    impl DAGCreator {
        fn new() -> Self {
            DAGCreator {
                dag: Graph::<NodeData, i32>::new(),
                current_node_id: 0,
            }
        }

        fn add_node(&mut self) -> NodeIndex {
            let node = NodeData::new(
                self.current_node_id,
                &self.current_node_id.to_string(),
                "0",
                1,
                Some(100),
            );
            self.current_node_id += 1;
            self.dag.add_node(node)
        }

        fn add_edge(&mut self, source: NodeIndex, target: NodeIndex) {
            self.dag.add_edge(source, target, 0);
        }

        fn get_dag(&self) -> &Graph<NodeData, i32> {
            &self.dag
        }
    }

    #[test]
    fn test_split_dag_into_chains_one_node() {
        let mut dag_creator = DAGCreator::new();
        dag_creator.add_node();

        let mut current_chain_priority = 0;
        let (_, chains) = split_dag_into_chains(
            &mut dag_creator.get_dag().clone(),
            &mut current_chain_priority,
        );

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].head_timer_callback.borrow().name, "0");
        assert_eq!(chains[0].regular_callbacks.len(), 0);
    }

    #[test]
    fn test_split_dag_into_chains_one_path() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);

        let mut current_chain_priority = 0;
        let (_, chains) = split_dag_into_chains(
            &mut dag_creator.get_dag().clone(),
            &mut current_chain_priority,
        );

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].head_timer_callback.borrow().name, "0");
        assert_eq!(chains[0].regular_callbacks.len(), 1);
    }

    #[test]
    fn test_split_dag_into_chains_branch() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        let v3 = dag_creator.add_node();
        let v4 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);
        dag_creator.add_edge(v1, v2);
        dag_creator.add_edge(v0, v3);
        dag_creator.add_edge(v3, v4);

        let mut current_chain_priority = 0;
        let (_, chains) = split_dag_into_chains(
            &mut dag_creator.get_dag().clone(),
            &mut current_chain_priority,
        );

        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].head_timer_callback.borrow().name, "0");
        assert_eq!(chains[0].regular_callbacks.len(), 2);
        assert_eq!(chains[1].regular_callbacks.len(), 1);
    }

    #[test]
    fn test_split_dag_into_chains_merge() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        dag_creator.add_edge(v0, v2);
        dag_creator.add_edge(v1, v2);

        let mut current_chain_priority = 0;
        let (_, chains) = split_dag_into_chains(
            &mut dag_creator.get_dag().clone(),
            &mut current_chain_priority,
        );

        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].regular_callbacks.len(), 1);
        assert_eq!(chains[1].regular_callbacks.len(), 0);
    }

    #[test]
    fn test_split_dag_into_chains_branch_and_merge() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        let v3 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);
        dag_creator.add_edge(v0, v2);
        dag_creator.add_edge(v1, v3);
        dag_creator.add_edge(v2, v3);

        let mut current_chain_priority = 0;
        let (_, chains) = split_dag_into_chains(
            &mut dag_creator.get_dag().clone(),
            &mut current_chain_priority,
        );

        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].regular_callbacks.len(), 2);
        assert_eq!(chains[1].regular_callbacks.len(), 0);
    }

    #[test]
    fn test_get_weakly_connected_graphs_one_node() {
        let mut dag_creator = DAGCreator::new();
        dag_creator.add_node();

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn test_get_weakly_connected_graphs_two_node() {
        let mut dag_creator = DAGCreator::new();
        dag_creator.add_node();
        dag_creator.add_node();

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_get_weakly_connected_graphs_one_path() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn test_get_weakly_connected_graphs_two_path() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        let v3 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);
        dag_creator.add_edge(v2, v3);

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_get_weakly_connected_graphs_branch() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);
        dag_creator.add_edge(v0, v2);

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn test_get_weakly_connected_graphs_merge() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        dag_creator.add_edge(v0, v2);
        dag_creator.add_edge(v1, v2);

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn test_get_weakly_connected_graphs_branch_and_merge() {
        let mut dag_creator = DAGCreator::new();
        let v0 = dag_creator.add_node();
        let v1 = dag_creator.add_node();
        let v2 = dag_creator.add_node();
        let v3 = dag_creator.add_node();
        dag_creator.add_edge(v0, v1);
        dag_creator.add_edge(v0, v2);
        dag_creator.add_edge(v1, v3);
        dag_creator.add_edge(v2, v3);

        let components = get_weakly_connected_graphs(dag_creator.get_dag());
        assert_eq!(components.len(), 1);
    }
}
