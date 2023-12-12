use crate::{callable::*, FlyString};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Function(Callable),
    String(FlyString),
}

impl Value {
    pub fn builtin(f: BuiltinFuntion) -> Self {
        f.into()
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::Function(_) => "function",
            Value::String(_) => "string",
        }
    }
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

impl From<BuiltinFuntion> for Value {
    fn from(value: BuiltinFuntion) -> Self {
        Self::Function(value.into())
    }
}

impl From<FunctionDescriptor> for Value {
    fn from(value: FunctionDescriptor) -> Self {
        Self::Function(value.into())
    }
}
