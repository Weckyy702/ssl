use std::{
    collections::{HashMap, VecDeque},
    iter::Peekable,
    num::ParseFloatError,
};

use thiserror::Error;

struct FunctionDescriptor {
    name: String,
    operations: Vec<Operation>,
}

#[derive(Debug, Clone)]
enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
enum Operation {
    // Stack operations
    Push(Value),
    Pop,
    Swap,
    Dup,
    Assign,
    PushFrom(String),
    // Math
    Add,
    Sub,
    Mul,
    Div,
    //Misc
    Print,
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Unknown character {0}")]
    UnknownCharacter(char),
    #[error("Invalid numeric literal {0}")]
    InvalidNumber(ParseFloatError),
    #[error("Must have an identifier after $")]
    InvalidPushFrom,
}

fn read_string<I: Iterator<Item = char>>(input: &mut Peekable<I>, c: Option<char>) -> String {
    let mut s = String::with_capacity(10);
    if let Some(c) = c {
        s.push(c);
    }
    while let Some(c) = input.peek() {
        if c.is_ascii_whitespace() {
            break;
        }
        s.push(*c);
        input.next();
    }
    s
}

fn parse_function<I: Iterator<Item = char>>(
    input: &mut Peekable<I>,
    name: String,
) -> Result<FunctionDescriptor, ParseError> {
    use Operation as O;

    let mut f = FunctionDescriptor {
        name,
        operations: vec![],
    };

    while let Some(c) = input.next() {
        let op = match c {
            c if c.is_ascii_whitespace() => continue,
            c if c.is_ascii_digit() => {
                let mut s = String::with_capacity(10);
                s.push(c);

                while let Some(c) = input.peek() {
                    if !c.is_ascii_digit() && *c != '.' {
                        break;
                    }
                    s.push(*c);
                    input.next();
                }

                s.parse()
                    .map(Value::Number)
                    .map(O::Push)
                    .map_err(ParseError::InvalidNumber)?
            }
            '+' => O::Add,
            '-' => O::Sub,
            '*' => O::Mul,
            '/' => O::Div,
            '!' => O::Pop,
            ':' => {
                if let Some('=') = input.peek() {
                    input.next();
                    O::Assign
                } else {
                    return Err(ParseError::UnknownCharacter(c));
                }
            }
            '$' => {
                let name = read_string(input, None);
                if name.is_empty() {
                    return Err(ParseError::InvalidPushFrom);
                }
                O::PushFrom(name)
            }
            '?' => O::Print,
            c => O::Push(Value::String(read_string(input, Some(c)))),
        };
        f.operations.push(op);
    }

    Ok(f)
}

fn parse(input: impl Iterator<Item = char>) -> Result<Vec<FunctionDescriptor>, ParseError> {
    let mut input = input.peekable();
    let mut functions = vec![];
    functions.push(parse_function(&mut input, "main".into())?);
    Ok(functions)
}

#[derive(Debug, Default)]
struct Scope {
    names: HashMap<String, Value>,
}

#[derive(Debug, Default)]
struct MachineState {
    scopes: VecDeque<Scope>,
    stack: VecDeque<Value>,
}

fn execute_function(MachineState { scopes, stack }: &mut MachineState, f: &FunctionDescriptor) {
    use Operation as O;
    use Value as V;

    macro_rules! pop_as {
        ($type:ident,$msg:literal) => {{
            let Some(V::$type(x)) = stack.pop_back() else {
                panic!($msg);
            };
            x
        }};
    }

    scopes.push_back(Scope::default());
    let current_scope = scopes.back_mut().unwrap();

    let mut i = 0;
    while let Some(op) = f.operations.get(i) {
        match op {
            O::Push(v) => stack.push_back(v.clone()),
            O::Pop => {
                stack.pop_back();
            }
            O::Swap => todo!(),
            O::Dup => todo!(),
            O::Assign => {
                let name = pop_as!(String, "Variable name must be a string");
                let Some(value) = stack.pop_back() else {
                    panic!("Must have a value to assign");
                };
                current_scope.names.insert(name, value);
            }
            O::PushFrom(name) => {
                let Some(value) = current_scope.names.get(name).cloned() else {
                    panic!("Name {name} is not bound");
                };
                stack.push_back(value);
            }
            O::Add => {
                let a = pop_as!(Number, "Cannot add non-numbers");
                let b = pop_as!(Number, "Cannot add non-numbers");
                stack.push_back(V::Number(a + b));
            }
            O::Sub => {
                let a = pop_as!(Number, "Cannot subtract non-numbers");
                let b = pop_as!(Number, "Cannot subtract non-numbers");
                stack.push_back(V::Number(a - b));
            }
            O::Mul => {
                let a = pop_as!(Number, "Cannot multiply non-numbers");
                let b = pop_as!(Number, "Cannot multiply non-numbers");
                stack.push_back(V::Number(a * b));
            }
            O::Div => {
                let a = pop_as!(Number, "Cannot divide non-numbers");
                let b = pop_as!(Number, "Cannot divide non-numbers");
                if b == 0. {
                    panic!("Attempt to divide by zero");
                }
                stack.push_back(V::Number(a / b));
            }
            O::Print => {
                println!("{:?}", stack.pop_back());
            }
        }
        i += 1;
    }

    scopes.pop_back();
}

fn main() -> Result<(), ParseError> {
    let input = "12 2 + x ??";

    let functions = parse(input.chars())?;

    for op in &functions[0].operations {
        println!("{:?}", op);
    }

    let mut state = MachineState::default();
    execute_function(&mut state, &functions[0]);

    println!("{:?}", state);

    Ok(())
}
