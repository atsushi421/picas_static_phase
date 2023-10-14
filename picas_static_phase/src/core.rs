use std::cell::RefCell;

use crate::executor::Executor;

pub struct Core<'a> {
    pub id: usize,
    allocated_executors: Vec<&'a RefCell<Executor<'a>>>,
}

impl<'a> Core<'a> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            allocated_executors: Vec::new(),
        }
    }

    pub fn get_utilization(&self) -> f64 {
        self.allocated_executors
            .iter()
            .map(|executor| executor.borrow().get_utilization())
            .sum()
    }

    pub fn allocate_executor(&mut self, executor: &'a RefCell<Executor<'a>>) {
        for cbg in &executor.borrow().allocated_callback_groups {
            cbg.borrow_mut().set_core_id(self.id);
        }
    }
}
