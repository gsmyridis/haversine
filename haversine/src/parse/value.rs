use super::Token;
use std::collections::HashMap;

/// It represents a JSON value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValueError {
    TryFromToken(Token),
    TryIntof64,
}

impl TryFrom<Token> for Value {
    type Error = ValueError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Null => Ok(Value::Null),
            Token::Bool(b) => Ok(Value::Bool(b)),
            Token::Number(n) => Ok(Value::Number(n)),
            Token::String(s) => Ok(Value::String(s)),
            _ => Err(ValueError::TryFromToken(token)),
        }
    }
}

impl TryInto<f64> for &Value {
    type Error = ValueError;

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(ValueError::TryIntof64),
        }
    }
}
