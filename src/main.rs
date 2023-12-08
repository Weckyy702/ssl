use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    iter::Peekable,
    num::ParseFloatError,
    rc::Rc,
};

use once_cell::sync::Lazy;

use thiserror::Error;

macro_rules! pop_as {
    ($state:ident,$type:ident) => {{
        let Value::$type(v) = $state.pop()? else {
            return Err(ExecuteError::TypeMismatch(FlyString::from_str(stringify!(
                $type
            ))));
        };
        v
    }};
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct FlyString(Rc<str>);

impl std::fmt::Debug for FlyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FlyString {
    fn from_string(s: String) -> Self {
        let strings = Self::interned_strings();

        if let Some(s) = strings.get(&s) {
            return Self(Rc::clone(s));
        }
        let s = Rc::clone(strings.entry(s.clone()).or_insert(s.into()));
        Self(s)
    }

    fn from_str(s: &str) -> Self {
        let strings = Self::interned_strings();

        if let Some(s) = strings.get(s) {
            return Self(Rc::clone(s));
        }
        let s = Rc::clone(strings.entry(s.into()).or_insert(s.into()));
        Self(s)
    }

    fn interned_strings() -> &'static mut HashMap<String, Rc<str>> {
        static mut STRINGS: Lazy<HashMap<String, Rc<str>>> = Lazy::new(HashMap::default);
        //SAFETY: we only create FlyStrings on one thread
        unsafe { &mut STRINGS }
    }
}

impl From<String> for FlyString {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl From<&str> for FlyString {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl Display for FlyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
struct FunctionDescriptor {
    operations: Vec<Operation>,
    num_args: usize,
}

type BuiltinFuntion = fn(&mut MachineState) -> Result<(), ExecuteError>;

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Number(f64),
    Function(Rc<FunctionDescriptor>),
    Builtin(BuiltinFuntion),
    String(FlyString),
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

#[derive(Debug, Clone)]
enum Operation {
    Push(Value),
    PushId(FlyString),
    PushRaw(FlyString),
    PushArg(usize),
    If(Vec<Operation>, Vec<Operation>),
    Return,
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Invalid numeric literal {0}")]
    InvalidNumber(ParseFloatError),
    #[error("Must have an identifier after $")]
    InvalidRawPush,
    #[error("Unclosed string literal")]
    InvalidString,
}

fn read_while<I, F>(input: &mut Peekable<I>, c: Option<char>, f: F) -> String
where
    I: Iterator<Item = char>,
    F: Fn(&char) -> bool,
{
    let mut s = String::with_capacity(10);
    if let Some(c) = c {
        s.push(c);
    }
    while let Some(c) = input.peek() {
        if !f(c) {
            break;
        }
        s.push(*c);
        input.next();
    }
    s
}

fn read_string<I: Iterator<Item = char>>(input: &mut Peekable<I>, c: Option<char>) -> String {
    read_while(input, c, |c| !c.is_ascii_whitespace())
}

fn parse_internal<I: Iterator<Item = char>>(
    input: &mut Peekable<I>,
) -> Result<FunctionDescriptor, ParseError> {
    use Operation as O;

    let mut f = FunctionDescriptor {
        operations: vec![],
        num_args: 0,
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
            '$' => {
                let name = read_string(input, None);
                if name.is_empty() {
                    return Err(ParseError::InvalidRawPush);
                }

                if let Ok(index) = name.parse::<usize>() {
                    f.num_args = usize::max(index + 1, f.num_args);
                    O::PushArg(index)
                } else {
                    O::PushRaw(name.into())
                }
            }
            '\'' => {
                let s = read_while(input, None, |c| !c.is_ascii_whitespace() && *c != '\'');
                let Some('\'') = input.next() else {
                    return Err(ParseError::InvalidString);
                };
                O::Push(Value::String(s.into()))
            }
            c => {
                let s = read_string(input, Some(c));
                match s.as_str() {
                    "end" => break,
                    "fn" => {
                        let f = parse_internal(input)?;
                        O::Push(Value::Function(f.into()))
                    }
                    "if" => {
                        let FunctionDescriptor {
                            operations,
                            num_args,
                        } = parse_internal(input)?;
                        f.num_args = usize::max(f.num_args, num_args);
                        O::If(operations, vec![])
                    }
                    "ret" => O::Return,
                    _ => O::PushId(s.into()),
                }
            }
        };
        f.operations.push(op);
    }

    Ok(f)
}

fn parse<I: Iterator<Item = char>>(input: I) -> Result<FunctionDescriptor, ParseError> {
    parse_internal(&mut input.peekable())
}

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

fn print(state: &mut MachineState) -> Result<(), ExecuteError> {
    use Value as V;
    match state.pop() {
        Ok(V::Bool(b)) => println!("{b}"),
        Ok(V::Number(x)) => println!("{x}"),
        Ok(V::String(s)) => println!("{s}"),
        Ok(V::Function(_)) | Ok(V::Builtin(_)) => println!("<function>"),
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

#[derive(Debug)]
struct Scope {
    names: HashMap<FlyString, Value>,
    args: Vec<Value>,
    inherits_from_parent: bool,
}

impl Scope {
    fn global(args: Vec<Value>) -> Self {
        Self {
            names: HashMap::from([
                ("+".into(), Value::Builtin(add)),
                ("-".into(), Value::Builtin(sub)),
                ("*".into(), Value::Builtin(mul)),
                ("/".into(), Value::Builtin(div)),
                ("<".into(), Value::Builtin(lt)),
                (".".into(), Value::Builtin(print)),
                (":=".into(), Value::Builtin(assign)),
            ]),
            args,
            inherits_from_parent: false,
        }
    }

    fn function(args: Vec<Value>) -> Self {
        Self {
            names: Default::default(),
            args,
            inherits_from_parent: false,
        }
    }

    fn conditional() -> Self {
        Self {
            names: Default::default(),
            args: Default::default(),
            inherits_from_parent: true,
        }
    }

    fn get(&self, id: &FlyString) -> Option<Value> {
        self.names.get(id).cloned()
    }

    fn set(&mut self, name: FlyString, value: Value) {
        self.names.insert(name, value);
    }

    fn get_arg(&self, index: usize) -> Option<Value> {
        self.args.get(index).cloned()
    }
}

#[derive(Debug, Default)]
struct MachineState {
    scopes: VecDeque<Scope>,
    stack: VecDeque<Value>,
}

impl MachineState {
    fn pop(&mut self) -> Result<Value, ExecuteError> {
        self.stack.pop_back().ok_or(ExecuteError::EmptyStack)
    }

    fn push(&mut self, value: Value) {
        self.stack.push_back(value)
    }

    fn global_scope(&self) -> &Scope {
        self.scopes.front().expect("Has global scope")
    }

    fn current_scope(&self) -> &Scope {
        self.scopes.back().expect("Has at least one scope")
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.back_mut().expect("Has at least one scope")
    }

    fn look_up(&self, name: &FlyString) -> Option<Value> {
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

    fn get_arg(&self, index: usize) -> Result<Value, ExecuteError> {
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
}

#[derive(Debug, Error)]
enum ExecuteError {
    #[error("Type mismatch: Expected {0}")]
    TypeMismatch(FlyString),
    #[error("Unbound identifier {0}")]
    UnboundIdentifier(FlyString),
    #[error("Tried to pop from empty stack")]
    EmptyStack,
    #[error("Unbound argument number {0}")]
    UnboundArgument(usize),
}

fn push_or_execute(state: &mut MachineState, v: Value) -> Result<(), ExecuteError> {
    use Value as V;
    match v {
        V::Function(f) => execute_function(state, &f)?,
        V::Builtin(f) => f(state)?,
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
            O::PushArg(index) => {
                push_or_execute(state, state.get_arg(*index)?)?;
            }
            O::If(if_body, else_body) => {
                let condition = pop_as!(state, Bool);
                if condition {
                    state.scopes.push_back(Scope::conditional());
                    let do_return = execute_function_code(state, &if_body)?;
                    state.scopes.pop_back();
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

fn execute_function(state: &mut MachineState, f: &FunctionDescriptor) -> Result<(), ExecuteError> {
    //TOOD: double allocation :(
    let args = (0..(f.num_args))
        .map(|_| state.pop())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .rev()
        .collect();
    state.scopes.push_back(Scope::function(args));
    execute_function_code(state, &f.operations)?;
    state.scopes.pop_back();
    Ok(())
}

fn execute(
    main_function: &FunctionDescriptor,
    input_args: Vec<Value>,
) -> Result<MachineState, ExecuteError> {
    let mut state = MachineState::default();
    state.scopes.push_back(Scope::global(input_args));
    execute_function_code(&mut state, &main_function.operations)?;
    Ok(state)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"
        fn 
            1 $0 < if
                0 ret
            end
            1 $0 - sum $0 +
        end 'sum' :=

        fn
            2 $0 < if
                $0 ret
            end
            1 $0 - fib 2 $0 - fib +
        end 'fib' :=

        25 fib .
    ";

    let function = parse(input.chars())?;

    /*for op in &function.operations {
        println!("{op:?}");
    }*/

    println!("{:?}", execute(&function, vec![1f64.into(), 2f64.into(),])?);

    Ok(())
}
