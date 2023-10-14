use std::cell::RefCell;

use crate::callback_group::CallbackGroup;

pub struct Executor<'a> {
    pub priority: i32,
    pub allocated_callback_groups: Vec<&'a RefCell<CallbackGroup>>,
}

impl<'a> Executor<'a> {
    pub fn new(priority: i32) -> Self {
        Self {
            priority,
            allocated_callback_groups: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.allocated_callback_groups.is_empty()
    }

    pub fn allocate_callback_group(&mut self, callback_group: &'a RefCell<CallbackGroup>) {
        callback_group
            .borrow_mut()
            .set_executor_priority(self.priority);
        self.allocated_callback_groups.push(callback_group);
    }

    pub fn get_utilization(&self) -> f64 {
        self.allocated_callback_groups
            .iter()
            .map(|callback_group| callback_group.borrow().get_utilization())
            .sum()
    }
}
