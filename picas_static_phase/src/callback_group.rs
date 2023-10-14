use std::{cell::RefCell, rc::Rc};

use crate::callback::Callback;

pub struct CallbackGroup {
    pub id: String,
    pub callbacks: Vec<Rc<RefCell<dyn Callback>>>,
    pub executor_priority: Option<i32>,
    pub core_id: Option<usize>,
}

impl CallbackGroup {
    pub fn new(id: &str, callbacks: Vec<Rc<RefCell<dyn Callback>>>) -> Self {
        Self {
            id: id.to_string(),
            callbacks,
            executor_priority: None,
            core_id: None,
        }
    }

    pub fn get_utilization(&self) -> f64 {
        self.callbacks
            .iter()
            .map(|callback| callback.borrow().get_utilization())
            .sum()
    }

    pub fn set_executor_priority(&mut self, executor_priority: i32) {
        self.executor_priority = Some(executor_priority);
    }

    pub fn get_executor_priority(&self) -> i32 {
        self.executor_priority.unwrap()
    }

    pub fn set_core_id(&mut self, core_id: usize) {
        self.core_id = Some(core_id);
    }

    pub fn get_core_id(&self) -> usize {
        self.core_id.unwrap()
    }
}
