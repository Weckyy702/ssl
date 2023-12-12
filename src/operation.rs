use crate::{FlyString, Value};

#[derive(Debug, Clone)]
pub enum Operation {
    Push(Value),
    PushId(FlyString),
    PushRaw(FlyString),
    PushArg(usize),
    If(Vec<Operation>, Vec<Operation>),
    Return,
}
