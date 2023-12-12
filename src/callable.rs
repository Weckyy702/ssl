use crate::{
    execute::{execute_function, ExecuteError},
    machine_state::MachineState,
    operation::Operation,
    value::Value,
    FlyString,
};

use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, Default)]
pub struct FunctionDescriptor {
    pub(crate) operations: Vec<Operation>,
    pub(crate) captured_names: HashMap<FlyString, Value>,
    pub(crate) num_args: usize,
}

pub type BuiltinFuntion = fn(&mut MachineState) -> Result<(), ExecuteError>;

#[derive(Debug, Clone)]
pub enum CallableKind {
    Function(Rc<FunctionDescriptor>),
    Builtin(BuiltinFuntion),
}

#[derive(Debug, Clone)]
pub struct Callable {
    pub(crate) kind: CallableKind,
    pub(crate) bound_arguments: Vec<Value>,
}

impl Callable {
    pub fn execute(&self, state: &mut MachineState) -> Result<(), ExecuteError> {
        match &self.kind {
            CallableKind::Builtin(f) => {
                self.bound_arguments
                    .iter()
                    .rev()
                    .cloned()
                    .for_each(|arg| state.push(arg));
                f(state)
            }
            CallableKind::Function(f) => execute_function(state, f, &self.bound_arguments),
        }
    }
}

impl From<BuiltinFuntion> for Callable {
    fn from(value: BuiltinFuntion) -> Self {
        Self {
            kind: CallableKind::Builtin(value),
            bound_arguments: vec![],
        }
    }
}

impl From<FunctionDescriptor> for Callable {
    fn from(value: FunctionDescriptor) -> Self {
        Self {
            kind: CallableKind::Function(value.into()),
            bound_arguments: vec![],
        }
    }
}
