use log::warn;
use petgraph::{
    algo::toposort,
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};
use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub struct NodeData {
    pub id: usize,
    pub name: String,
    pub callback_group_id: String,
    pub wcet: i32,
    pub period: Option<i32>,
    pub temp_params: HashMap<String, i32>,
}

impl NodeData {
    pub fn new(
        id: usize,
        name: &str,
        callback_group_id: &str,
        wcet: i32,
        period: Option<i32>,
    ) -> Self {
        NodeData {
            id,
            name: name.to_string(),
            callback_group_id: callback_group_id.to_string(),
            wcet,
            period,
            temp_params: HashMap::new(),
        }
    }
}

pub trait GraphExtension {
    fn update_temp_param(&mut self, node_i: NodeIndex, key: &str, value: i32);
    fn add_dummy_source_node(&mut self) -> NodeIndex;
    fn add_dummy_sink_node(&mut self) -> NodeIndex;
    fn remove_dummy_nodes(&mut self);
    fn remove_nodes(&mut self, node_indices: &[NodeIndex]);
    fn calculate_earliest_start_times(&mut self);
    fn calculate_latest_start_times(&mut self);
    fn get_critical_path(&mut self) -> Vec<NodeIndex>;
    fn get_source_nodes(&self) -> Vec<NodeIndex>;
    fn get_sink_nodes(&self) -> Vec<NodeIndex>;
    fn get_head_period(&self) -> Option<i32>;
}

impl GraphExtension for Graph<NodeData, i32> {
    fn update_temp_param(&mut self, node_i: NodeIndex, key: &str, value: i32) {
        let target_node = self.node_weight_mut(node_i).unwrap();
        target_node.temp_params.insert(key.to_string(), value);
    }

    fn add_dummy_source_node(&mut self) -> NodeIndex {
        let source_nodes = self.get_source_nodes();
        let dummy_source_i =
            self.add_node(NodeData::new(self.node_count(), "dummy", "dummy", 0, None));

        for source_i in source_nodes {
            self.add_edge(dummy_source_i, source_i, 0);
        }
        dummy_source_i
    }

    fn add_dummy_sink_node(&mut self) -> NodeIndex {
        let sink_nodes = self.get_sink_nodes();
        let dummy_sink_i =
            self.add_node(NodeData::new(self.node_count(), "dummy", "dummy", 0, None));
        for sink_i in sink_nodes {
            self.add_edge(sink_i, dummy_sink_i, 0);
        }
        dummy_sink_i
    }

    fn remove_dummy_nodes(&mut self) {
        let dummy_nodes: Vec<_> = self
            .node_indices()
            .filter(|&node_i| self[node_i].name == "dummy")
            .collect();

        for node_i in dummy_nodes {
            self.remove_node(node_i);
        }
    }

    fn remove_nodes(&mut self, node_indices: &[NodeIndex]) {
        for node_i in node_indices.iter().rev() {
            self.remove_node(*node_i);
        }
    }

    /// Calculate the earliest start times for each node in the DAG.
    fn calculate_earliest_start_times(&mut self) {
        let mut earliest_start_times = vec![0; self.node_count()];

        let sorted_nodes = toposort(&*self, None).unwrap();
        for node_i in sorted_nodes {
            let max_earliest_start_time = self
                .edges_directed(node_i, Incoming)
                .map(|edge| {
                    let source_node = edge.source();
                    let exe_time = self[source_node].wcet;
                    earliest_start_times[source_node.index()] + exe_time
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0);

            earliest_start_times[node_i.index()] = max_earliest_start_time;
            self.update_temp_param(node_i, "earliest_start_time", max_earliest_start_time);
        }
        assert!(
            !earliest_start_times.iter().any(|&time| time < 0),
            "The earliest start times should be non-negative."
        );
    }

    /// Calculate the latest start times for each node in the DAG.
    fn calculate_latest_start_times(&mut self) {
        self.calculate_earliest_start_times();
        let sorted_nodes = toposort(&*self, None).unwrap();
        let mut latest_start_times = vec![i32::MAX; self.node_count()];
        let sink_node_index = self.get_sink_nodes();
        latest_start_times[sink_node_index[0].index()] =
            self[sink_node_index[0]].temp_params["earliest_start_time"];

        for &node_i in sorted_nodes.iter().rev() {
            let min_latest_start_time = self
                .edges_directed(node_i, Outgoing)
                .map(|edge| {
                    let target_node = edge.target();
                    let pre_exe_time = self[node_i].wcet;
                    latest_start_times[target_node.index()] - pre_exe_time
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(self[sink_node_index[0]].temp_params["earliest_start_time"]);

            latest_start_times[node_i.index()] = min_latest_start_time;
            self.update_temp_param(node_i, "latest_start_time", min_latest_start_time);
        }

        assert!(
            !latest_start_times.iter().any(|&time| time < 0),
            "The latest start times should be non-negative."
        );
    }

    fn get_critical_path(&mut self) -> Vec<NodeIndex> {
        self.add_dummy_sink_node();
        let start_node = self.add_dummy_source_node();
        self.calculate_earliest_start_times();
        self.calculate_latest_start_times();
        let mut path_search_queue = VecDeque::new();
        path_search_queue.push_back((start_node, vec![start_node]));
        let mut critical_path = Vec::new();

        while let Some((node, mut current_critical_path)) = path_search_queue.pop_front() {
            let outgoing_edges: Vec<_> = self.edges_directed(node, Outgoing).collect();

            if outgoing_edges.is_empty() {
                current_critical_path.pop(); // Remove the dummy sink node
                current_critical_path.remove(0); // Remove the dummy source node
                critical_path.push(current_critical_path);
            } else {
                for edge in outgoing_edges {
                    let target_node = edge.target();
                    if self[target_node].temp_params["earliest_start_time"]
                        == self[target_node].temp_params["latest_start_time"]
                    {
                        let mut new_critical_path = current_critical_path.clone();
                        new_critical_path.push(target_node);
                        path_search_queue.push_back((target_node, new_critical_path));
                    }
                }
            }
        }

        self.remove_dummy_nodes();

        if critical_path.len() > 1 {
            warn!("There are more than one critical paths.");
        }

        // Reset the temp_params
        for node in self.node_indices() {
            self[node].temp_params.clear();
        }

        critical_path[0].clone()
    }

    fn get_source_nodes(&self) -> Vec<NodeIndex> {
        self.node_indices()
            .filter(|&i| self.edges_directed(i, Incoming).next().is_none())
            .collect::<Vec<_>>()
    }

    fn get_sink_nodes(&self) -> Vec<NodeIndex> {
        self.node_indices()
            .filter(|&i| self.edges_directed(i, Outgoing).next().is_none())
            .collect::<Vec<_>>()
    }

    fn get_head_period(&self) -> Option<i32> {
        self[self.get_source_nodes()[0]].period
    }
}
