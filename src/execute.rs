use crate::{
    callable::*, machine_state::MachineState, operation::Operation, scope::Scope, FlyString, Value,
};

use std::collections::VecDeque;

use thiserror::Error;

#[macro_export]
macro_rules! pop_as {
    ($state:ident,$type:ident) => {{
        let Value::$type(v) = $state.pop()? else {
            return Err(ExecuteError::TypeMismatch(stringify!($type).into()));
        };
        v
    }};
}

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("Type mismatch: Expected {0}")]
    TypeMismatch(FlyString),
    #[error("Unbound identifier {0}")]
    UnboundIdentifier(FlyString),
    #[error("Tried to pop from empty stack")]
    EmptyStack,
    #[error("Unbound argument number {0}")]
    UnboundArgument(usize),
    #[error("Type assertion failed: expected {0}, got {1}")]
    InvalidType(&'static str, FlyString),
    #[error("Tried to bind too many arguments")]
    TooManyBoundArgs,
}

fn push_or_execute(state: &mut MachineState, v: Value) -> Result<(), ExecuteError> {
    use Value as V;
    match v {
        V::Function(f) => f.execute(state)?,
        _ => state.push(v),
    }
    Ok(())
}

fn execute_function_code(
    state: &mut MachineState,
    operations: &Vec<Operation>,
) -> Result<bool, ExecuteError> {
    use Operation as O;

    let mut i = 0;
    while let Some(op) = operations.get(i) {
        //println!("{op:?}");
        match op {
            O::Push(v) => state.push(v.clone()),
            O::PushId(id) => {
                if let Some(v) = state.look_up(&id) {
                    push_or_execute(state, v)?;
                } else if let Some(v) = state.global_scope().get(&id) {
                    push_or_execute(state, v)?;
                } else {
                    return Err(ExecuteError::UnboundIdentifier(id.clone()));
                }
            }
            O::PushRaw(id) => {
                if let Some(v) = state.look_up(&id) {
                    state.push(v);
                } else if let Some(v) = state.global_scope().get(&id) {
                    state.push(v);
                } else {
                    return Err(ExecuteError::UnboundIdentifier(id.clone()));
                }
            }
            O::PushArg(index) => state.push(state.get_arg(*index)?),
            O::If(if_body, else_body) => {
                let condition = pop_as!(state, Bool);
                if condition {
                    state.push_scope(Scope::conditional());
                    let do_return = execute_function_code(state, &if_body)?;
                    state.pop_scope();
                    if do_return {
                        return Ok(true);
                    }
                } else {
                    assert!(else_body.len() == 0);
                }
            }
            O::Return => return Ok(true),
        }
        i += 1;
    }

    Ok(false)
}

pub(crate) fn execute_function(
    state: &mut MachineState,
    f: &FunctionDescriptor,
    bound_args: &Vec<Value>,
) -> Result<(), ExecuteError> {
    let mut args = VecDeque::default();

    let args_to_pop = f.num_args - bound_args.len();
    for _ in 0..args_to_pop {
        args.push_front(state.pop()?);
    }

    bound_args
        .iter()
        .rev()
        .cloned()
        .for_each(|x| args.push_front(x));

    state.push_scope(Scope::function(args.into(), f.captured_names.clone()));
    execute_function_code(state, &f.operations)?;
    state.pop_scope();
    Ok(())
}

pub fn execute(
    main_function: &FunctionDescriptor,
    input_args: Vec<Value>,
) -> Result<MachineState, ExecuteError> {
    let mut state = MachineState::default();
    state.push_scope(Scope::global(input_args));
    execute_function_code(&mut state, &main_function.operations)?;
    Ok(state)
}
