use clap::Parser;
use picas_static_phase::core::Core;
use picas_static_phase::dag_parser::parse_dags;
use picas_static_phase::executor::Executor;
use picas_static_phase::export_config::export_config;
use picas_static_phase::picas_helper::{
    assign_cb_prio, find_highest_prio_empty_executor, meets_strategy_5_or_6,
    sort_cbgs_by_highest_cb_prio, sort_cores_by_utilization,
};
use std::cell::RefCell;

#[derive(Parser)]
#[clap(name = "PiCAS static phase", version = "1.0", about = "")]
struct ArgParser {
    /// Path to DAGSet directory.
    #[clap(short = 'd', long = "dag_dir", default_value = "../autoware_dags")]
    dag_dir_path: String,
    /// Number of processing cores.
    #[clap(short = 'c', long = "number_of_cores", default_value = "16")]
    number_of_cores: usize,
    /// Path to output directory.
    #[clap(short = 'o', long = "output_path", default_value = "../")]
    output_dir_path: String,
}

const MAX_EXECUTORS: usize = 99;

fn main() {
    let arg: ArgParser = ArgParser::parse();

    let (mut callback_groups, mut chains) = parse_dags(&arg.dag_dir_path);
    let mut executors = Vec::with_capacity(MAX_EXECUTORS);
    for i in 0..MAX_EXECUTORS {
        executors.push(RefCell::new(Executor::new(i as i32)));
    }
    let mut cores = Vec::with_capacity(arg.number_of_cores);
    for i in 0..arg.number_of_cores {
        cores.push(Core::new(i));
    }

    // Starting PiCAS framework.
    assign_cb_prio(&mut chains);
    sort_cbgs_by_highest_cb_prio(&mut callback_groups);

    'outer: for callback_group in &callback_groups {
        if let Some(executor_i) = find_highest_prio_empty_executor(&executors) {
            // Part A
            let executor = &executors[executor_i];
            executor
                .borrow_mut()
                .allocate_callback_group(callback_group);

            sort_cores_by_utilization(&mut cores);
            for core in cores.iter_mut() {
                if executor.borrow().get_utilization() + core.get_utilization() <= 1.0 {
                    if meets_strategy_5_or_6(core, executor) {
                        core.allocate_executor(executor);
                        continue 'outer;
                    } else {
                        unreachable!(); // Autoware always meets strategies 5 and 6
                    }
                }
            }

            // Part C
            cores[0].allocate_executor(executor);
            // All cores meet strategies 5 and 6, so there is no need to merge executors.
        } else {
            // Part B
            unimplemented!(); // Part B is only reached if more than 99 executors are required.
        }
    }

    export_config(&arg.output_dir_path, &callback_groups)
}
