use crate::callable::FunctionDescriptor;
use crate::operation::Operation;
use crate::Value;

use std::{iter::Peekable, num::ParseFloatError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid numeric literal {0}")]
    InvalidNumber(ParseFloatError),
    #[error("Must have an identifier after $")]
    InvalidRawPush,
    #[error("Unclosed string literal")]
    InvalidString,
}

pub fn parse<I>(input: I) -> Result<FunctionDescriptor, ParseError>
where
    I: Iterator<Item = char>,
{
    parse_internal(&mut input.peekable())
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

fn read_string<I>(input: &mut Peekable<I>, c: Option<char>) -> String
where
    I: Iterator<Item = char>,
{
    read_while(input, c, |c| !c.is_ascii_whitespace())
}

fn parse_internal<I>(input: &mut Peekable<I>) -> Result<FunctionDescriptor, ParseError>
where
    I: Iterator<Item = char>,
{
    use Operation as O;

    let mut f = FunctionDescriptor::default();

    while let Some(c) = input.next() {
        let op = match c {
            c if c.is_ascii_whitespace() => continue,
            c if c.is_ascii_digit() => {
                let s = read_while(input, Some(c), |c| c.is_ascii_digit() || *c == '.');
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
                O::Push(s.into())
            }
            c => {
                let s = read_string(input, Some(c));
                match s.as_str() {
                    "end" => break,
                    "fn" => {
                        let f = parse_internal(input)?;
                        O::Push(f.into())
                    }
                    "if" => {
                        let FunctionDescriptor {
                            operations,
                            num_args,
                            ..
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
