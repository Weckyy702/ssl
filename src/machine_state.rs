use crate::{execute::ExecuteError, scope::Scope, FlyString, Value};

use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct MachineState {
    scopes: VecDeque<Scope>,
    stack: VecDeque<Value>,
}

impl MachineState {
    pub fn pop(&mut self) -> Result<Value, ExecuteError> {
        self.stack.pop_back().ok_or(ExecuteError::EmptyStack)
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push_back(value)
    }

    pub fn global_scope(&self) -> &Scope {
        self.scopes.front().expect("Has global scope")
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes.back().expect("Has at least one scope")
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.back_mut().expect("Has at least one scope")
    }

    pub fn look_up(&self, name: &FlyString) -> Option<Value> {
        let mut scopes = self.scopes.iter().rev();
        while let Some(scope) = scopes.next() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
            if !scope.inherits_from_parent {
                break;
            }
        }
        None
    }

    pub fn get_arg(&self, index: usize) -> Result<Value, ExecuteError> {
        let mut scopes = self.scopes.iter().rev();
        while let Some(scope) = scopes.next() {
            if let Some(var) = scope.get_arg(index) {
                return Ok(var);
            }
            if !scope.inherits_from_parent {
                break;
            }
        }
        Err(ExecuteError::UnboundArgument(index))
    }

    pub fn push_scope(&mut self, s: Scope) {
        self.scopes.push_back(s)
    }

    pub fn pop_scope(&mut self) -> Scope {
        self.scopes.pop_back().expect("Has scope")
    }
}
