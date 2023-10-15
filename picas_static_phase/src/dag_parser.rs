use crate::callback::{RegularCallback, TimerCallback};
use crate::callback_group::CallbackGroup;
use crate::chain::Chain;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use crate::graph_extension::{GraphExtension, NodeData};

use petgraph::{graph::Graph, prelude::*};
use std::path::PathBuf;
use yaml_rust::YamlLoader;

fn load_yaml(file_path: &str) -> Vec<yaml_rust::Yaml> {
    if !file_path.ends_with(".yaml") && !file_path.ends_with(".yml") {
        panic!("Invalid file type: {}", file_path);
    }
    let file_content = fs::read_to_string(file_path).unwrap();
    YamlLoader::load_from_str(&file_content).unwrap()
}

fn create_dag_from_yaml(file_path: &str) -> Graph<NodeData, i32> {
    let yaml_doc = load_yaml(file_path)[0];
    let mut dag = Graph::<NodeData, i32>::new();

    // add nodes to dag
    for node in yaml_doc["nodes"].as_vec().unwrap() {
        let mut id = "0".to_string();
        let mut callback_group_id = "0".to_string();
        let mut wcet = 0;
        let mut period = None;
        for (key, value) in node.as_hash().unwrap() {
            let key_str = key.as_str().unwrap();
            if key_str == "id" {
                id = value.as_str().unwrap().to_string();
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
        dag.add_node(NodeData::new(id, callback_group_id, wcet, period));
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

fn get_yaml_paths_from_dir(dir_path: &str) -> Vec<String> {
    if !std::fs::metadata(dir_path).unwrap().is_dir() {
        panic!("Not a directory");
    }
    let mut file_path_list = Vec::new();
    for dir_entry_result in PathBuf::from(dir_path).read_dir().unwrap() {
        let path = dir_entry_result.unwrap().path();
        let extension = path.extension().unwrap();
        if extension == "yaml" || extension == "yml" {
            file_path_list.push(path.to_str().unwrap().to_string());
        }
    }
    if file_path_list.is_empty() {
        panic!("No YAML file found in {}", dir_path);
    }
    file_path_list
}

/// load yaml files and return a DAGSet (dag list)
///
/// # Arguments
///
/// *  `dir_path` - dir path for yaml files
///
/// # Returns
///
/// *  `dag_set` - dag list (petgraph vector)
///
/// # Example
///
/// ```
/// use lib::dag_creator::create_dag_set_from_dir;
/// let dag_set = create_dag_set_from_dir("tests/sample_dags/multiple_yaml");
/// let first_node_num = dag_set[0].node_count();
/// let first_edge_num = dag_set[0].edge_count();
/// let first_node_exe_time = dag_set[0][dag_set[0].node_indices().next().unwrap()].params["execution_time"];
/// ```
pub fn create_dag_set_from_dir(dir_path: &str) -> Vec<Graph<NodeData, i32>> {
    let mut file_path_list = get_yaml_paths_from_dir(dir_path);
    file_path_list.sort();

    let mut dag_set: Vec<Graph<NodeData, i32>> = Vec::new();
    for (dag_id, file_path) in file_path_list.iter().enumerate() {
        let mut dag = create_dag_from_yaml(file_path);
        dag.set_dag_param("dag_id", dag_id as i32);
        dag_set.push(dag);
    }
    dag_set
}

pub fn parse_dag() -> (Vec<RefCell<CallbackGroup>>, Vec<Chain>) {
    // [chain0] timer0 -> regular0 -> regular1
    let timer0 = Rc::new(RefCell::new(TimerCallback::new(5, 40)));
    let regular0 = Rc::new(RefCell::new(RegularCallback::new(10)));
    let regular1 = Rc::new(RefCell::new(RegularCallback::new(10)));
    let callback_group0 = RefCell::new(CallbackGroup::new("cbg0", vec![timer0.clone()]));
    let callback_group1 = RefCell::new(CallbackGroup::new(
        "cbg1",
        vec![regular0.clone(), regular1.clone()],
    ));
    let chain0 = Chain::new(0, timer0.clone(), vec![regular0.clone(), regular1.clone()]);

    // [chain1] timer1 -> regular2 -> regular3 -> regular4 -> regular5
    let timer1 = Rc::new(RefCell::new(TimerCallback::new(5, 40)));
    let regular2 = Rc::new(RefCell::new(RegularCallback::new(15)));
    let regular3 = Rc::new(RefCell::new(RegularCallback::new(8)));
    let regular4 = Rc::new(RefCell::new(RegularCallback::new(1)));
    let regular5 = Rc::new(RefCell::new(RegularCallback::new(4)));
    let callback_group2 = RefCell::new(CallbackGroup::new("cbg2", vec![timer1.clone()]));
    let callback_group3 = RefCell::new(CallbackGroup::new(
        "cbg3",
        vec![regular2.clone(), regular3.clone()],
    ));
    let callback_group4 = RefCell::new(CallbackGroup::new(
        "cbg4",
        vec![regular4.clone(), regular5.clone()],
    ));
    let chain1 = Chain::new(
        1,
        timer1.clone(),
        vec![
            regular2.clone(),
            regular3.clone(),
            regular4.clone(),
            regular5.clone(),
        ],
    );

    // [chain2] timer2 -> regular6 -> regular7 -> regular8 -> regular9
    let timer2 = Rc::new(RefCell::new(TimerCallback::new(25, 100)));
    let regular6 = Rc::new(RefCell::new(RegularCallback::new(6)));
    let regular7 = Rc::new(RefCell::new(RegularCallback::new(14)));
    let regular8 = Rc::new(RefCell::new(RegularCallback::new(2)));
    let regular9 = Rc::new(RefCell::new(RegularCallback::new(9)));
    let callback_group5 = RefCell::new(CallbackGroup::new("cbg5", vec![timer2.clone()]));
    let callback_group6 = RefCell::new(CallbackGroup::new(
        "cbg6",
        vec![regular6.clone(), regular7.clone()],
    ));
    let callback_group7 = RefCell::new(CallbackGroup::new(
        "cbg7",
        vec![regular8.clone(), regular9.clone()],
    ));
    let chain2 = Chain::new(
        2,
        timer2.clone(),
        vec![
            regular6.clone(),
            regular7.clone(),
            regular8.clone(),
            regular9.clone(),
        ],
    );

    (
        vec![
            callback_group0,
            callback_group1,
            callback_group2,
            callback_group3,
            callback_group4,
            callback_group5,
            callback_group6,
            callback_group7,
        ],
        vec![chain0, chain1, chain2],
    )
}
