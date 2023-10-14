use crate::callback::{RegularCallback, TimerCallback};
use crate::callback_group::CallbackGroup;
use crate::chain::Chain;
use std::cell::RefCell;
use std::rc::Rc;

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
