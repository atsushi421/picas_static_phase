pub trait Callback {
    fn get_utilization(&self) -> f64;
    fn set_priority_within_executor(&mut self, priority_within_executor: i32);
    fn get_priority_within_executor(&self) -> i32;
}

macro_rules! getset_callback {
    () => {
        fn set_priority_within_executor(&mut self, priority_within_executor: i32) {
            self.priority_within_executor = Some(priority_within_executor);
        }

        fn get_priority_within_executor(&self) -> i32 {
            self.priority_within_executor.unwrap()
        }
    };
}

pub struct TimerCallback {
    wcet: i32,
    pub period: i32,
    priority_within_executor: Option<i32>,
}

impl Callback for TimerCallback {
    getset_callback!();

    fn get_utilization(&self) -> f64 {
        self.wcet as f64 / self.period as f64
    }
}

impl TimerCallback {
    pub fn new(wcet: i32, period: i32) -> Self {
        Self {
            wcet,
            period,
            priority_within_executor: None,
        }
    }
}

pub struct RegularCallback {
    wcet: i32,
    assumed_period: Option<i32>,
    priority_within_executor: Option<i32>,
}

impl Callback for RegularCallback {
    getset_callback!();

    fn get_utilization(&self) -> f64 {
        self.wcet as f64 / self.assumed_period.unwrap() as f64
    }
}

impl RegularCallback {
    pub fn new(wcet: i32) -> Self {
        Self {
            wcet,
            assumed_period: None,
            priority_within_executor: None,
        }
    }

    pub fn set_assumed_period(&mut self, assumed_period: i32) {
        self.assumed_period = Some(assumed_period);
    }

    pub fn get_assumed_period(&self) -> i32 {
        self.assumed_period.unwrap()
    }
}
