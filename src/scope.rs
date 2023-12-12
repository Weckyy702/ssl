use crate::{builtins::get_builtins, FlyString, Value};

use std::collections::HashMap;

#[derive(Debug)]
pub struct Scope {
    names: HashMap<FlyString, Value>,
    args: Vec<Value>,
    pub(crate) inherits_from_parent: bool,
}

impl Scope {
    pub fn global(args: Vec<Value>) -> Self {
        Self {
            names: get_builtins(),
            args,
            inherits_from_parent: false,
        }
    }

    pub fn function(args: Vec<Value>, captured_names: HashMap<FlyString, Value>) -> Self {
        Self {
            names: captured_names,
            args,
            inherits_from_parent: false,
        }
    }

    pub fn conditional() -> Self {
        Self {
            names: Default::default(),
            args: Default::default(),
            inherits_from_parent: true,
        }
    }

    pub fn names(&self) -> &HashMap<FlyString, Value> {
        &self.names
    }

    pub fn get(&self, id: &FlyString) -> Option<Value> {
        self.names.get(id).cloned()
    }

    pub fn set(&mut self, name: FlyString, value: Value) {
        self.names.insert(name, value);
    }

    pub fn get_arg(&self, index: usize) -> Option<Value> {
        self.args.get(index).cloned()
    }
}
