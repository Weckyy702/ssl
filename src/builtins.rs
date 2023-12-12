use crate::{
    callable::*, execute::ExecuteError, machine_state::MachineState, pop_as, FlyString, Value,
};

use std::collections::HashMap;

macro_rules! numeric_biop_impl {
    ($name:ident, $op:tt, $output:ident) => {
        fn $name(state: &mut MachineState) -> Result<(), ExecuteError> {
            use Value as V;
            let a = pop_as!(state, Number);
            let b = pop_as!(state, Number);
            state.push(V::$output(a $op b));
            Ok(())
        }
    };
}

numeric_biop_impl!(add, +, Number);
numeric_biop_impl!(sub, -, Number);
numeric_biop_impl!(mul, *, Number);
numeric_biop_impl!(div, /, Number);

numeric_biop_impl!(lt, <, Bool);

fn print_function(f: &FunctionDescriptor) {
    if f.captured_names.is_empty() {
        print!("<function");
        return;
    }
    print!("<closure: ");
    for (i, (name, _)) in f.captured_names.iter().enumerate() {
        if i != 0 {
            print!(", ");
        }
        print!("{name}");
    }
}

fn print_callable(f: &Callable) {
    match &f.kind {
        CallableKind::Builtin(_) => print!("<builtin"),
        CallableKind::Function(f) => print_function(&f),
    }
    if f.bound_arguments.is_empty() {
        println!(">");
        return;
    }

    print!(", bound arguments: ");
    for (i, val) in f.bound_arguments.iter().enumerate() {
        if i != 0 {
            print!(", ");
        }
        print!("${i}: {val:?}");
    }
    println!(">");
}

fn print(state: &mut MachineState) -> Result<(), ExecuteError> {
    use Value as V;
    match state.pop() {
        Ok(V::Bool(b)) => println!("{b}"),
        Ok(V::Number(x)) => println!("{x}"),
        Ok(V::String(s)) => println!("{s}"),
        Ok(V::Function(ref f)) => print_callable(f),
        Err(_) => println!("<empty>"),
    }
    Ok(())
}

fn assign(state: &mut MachineState) -> Result<(), ExecuteError> {
    let name = pop_as!(state, String);
    let value = state.pop()?;

    state.current_scope_mut().set(name, value);

    Ok(())
}

fn assert_type(state: &mut MachineState) -> Result<(), ExecuteError> {
    let type_name = pop_as!(state, String);
    let value = state.pop()?;

    if type_name != value.type_name() {
        return Err(ExecuteError::InvalidType(value.type_name(), type_name));
    }
    Ok(())
}

fn make_closure(state: &mut MachineState) -> Result<(), ExecuteError> {
    let Callable {
        kind,
        bound_arguments,
    } = pop_as!(state, Function);

    let f = match &kind {
        CallableKind::Function(f) => f,
        CallableKind::Builtin(_) => {
            return Err(ExecuteError::InvalidType("builtin", "function".into()))
        }
    };

    let captured_names = state.current_scope().names().clone();
    let closure = FunctionDescriptor {
        captured_names,
        num_args: f.num_args,
        operations: f.operations.clone(),
    };

    state.push(Value::Function(Callable {
        kind: CallableKind::Function(closure.into()),
        bound_arguments,
    }));

    Ok(())
}

fn bind(state: &mut MachineState) -> Result<(), ExecuteError> {
    let f = pop_as!(state, Function);
    let num_to_bind = pop_as!(state, Number) as usize;

    if let CallableKind::Function(ref f) = f.kind {
        if f.num_args < num_to_bind {
            return Err(ExecuteError::TooManyBoundArgs);
        }
    }

    let bound_arguments = (0..num_to_bind)
        .map(|_| state.pop())
        .collect::<Result<_, _>>()?;

    state.push(Value::Function(Callable {
        bound_arguments,
        ..f
    }));

    Ok(())
}

pub fn get_builtins() -> HashMap<FlyString, Value> {
    HashMap::from([
        ("+".into(), Value::builtin(add)),
        ("-".into(), Value::builtin(sub)),
        ("*".into(), Value::builtin(mul)),
        ("/".into(), Value::builtin(div)),
        ("<".into(), Value::builtin(lt)),
        (".".into(), Value::builtin(print)),
        (":=".into(), Value::builtin(assign)),
        ("!".into(), Value::builtin(assert_type)),
        ("^".into(), Value::builtin(make_closure)),
        ("bind".into(), Value::builtin(bind)),
    ])
}
