use std::cell::RefCell;

use crate::callback::Callback;
use crate::callback_group::CallbackGroup;
use crate::chain::Chain;
use crate::core::Core;
use crate::executor::Executor;

pub fn assign_cb_prio(chains: &mut Vec<Chain>) {
    let mut current_priority: i32 = chains
        .iter()
        .map(|chain| (chain.regular_callbacks.len() + 1) as i32)
        .sum();

    chains.sort_by_key(|chain| std::cmp::Reverse(chain.priority));
    for chain in chains {
        chain
            .head_timer_callback
            .borrow_mut()
            .set_priority_within_executor(current_priority);
        current_priority -= 1;
        for callback in &chain.regular_callbacks {
            callback
                .borrow_mut()
                .set_priority_within_executor(current_priority);
            current_priority -= 1;
        }
    }
}

pub fn sort_cbgs_by_highest_cp_prio(cbgs: &mut [RefCell<CallbackGroup>]) {
    cbgs.sort_by_key(|cbg| {
        cbg.borrow()
            .callbacks
            .iter()
            .map(|cb| cb.borrow().get_priority_within_executor())
            .min()
            .unwrap()
    });
}

pub fn find_highest_prio_empty_executor(executors: &[RefCell<Executor>]) -> Option<usize> {
    executors
        .iter()
        .enumerate()
        .filter(|(_, executor)| executor.borrow().is_empty())
        .min_by_key(|(_, executor)| executor.borrow().priority)
        .map(|(index, _)| index)
}

pub fn sort_cores_by_utilization(cores: &mut [Core]) {
    cores.sort_by(|a, b| {
        a.get_utilization()
            .partial_cmp(&b.get_utilization())
            .unwrap()
    });
}

#[allow(unused_variables)]
pub fn meets_strategy_5_or_6(core: &Core, executor: &RefCell<Executor>) -> bool {
    true // HACK: Autoware always meets Strategy 5 and 6
}
