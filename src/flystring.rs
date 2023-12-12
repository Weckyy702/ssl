use std::{collections::HashMap, fmt::Display, rc::Rc};

use once_cell::unsync::Lazy;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FlyString(Rc<str>);

impl std::fmt::Debug for FlyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
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

impl PartialEq<&str> for FlyString {
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}

impl Display for FlyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
