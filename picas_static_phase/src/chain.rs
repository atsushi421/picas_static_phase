use std::{cell::RefCell, rc::Rc};

use crate::callback::{RegularCallback, TimerCallback};

pub struct Chain {
    pub priority: i32,
    pub head_timer_callback: Rc<RefCell<TimerCallback>>,
    pub regular_callbacks: Vec<Rc<RefCell<RegularCallback>>>,
}

impl Chain {
    pub fn new(
        priority: i32,
        head_timer_callback: Rc<RefCell<TimerCallback>>,
        regular_callbacks: Vec<Rc<RefCell<RegularCallback>>>,
    ) -> Self {
        let period = head_timer_callback.borrow().period;
        for regular_callback in &regular_callbacks {
            regular_callback.borrow_mut().set_assumed_period(period)
        }
        Self {
            priority,
            head_timer_callback,
            regular_callbacks,
        }
    }
}
