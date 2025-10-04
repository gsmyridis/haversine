use std::collections::HashMap;

use super::parser::ParsingError;
use super::{Parser, Token, Value};

#[test]
fn test_null() {
    let parser = Parser::new("null");
    assert_eq!(Ok(Some(Value::Null)), parser.parse());
}

#[test]
fn test_true() {
    let parser = Parser::new("true");
    assert_eq!(Ok(Some(Value::Bool(true))), parser.parse());
}

#[test]
fn test_false() {
    let parser = Parser::new("false");
    assert_eq!(Ok(Some(Value::Bool(false))), parser.parse());
}

#[test]
fn test_string() {
    let parser = Parser::new("\"Hello, this is a string!\"");
    let string = Value::String("Hello, this is a string!".into());
    assert_eq!(Ok(Some(string)), parser.parse());
}

#[test]
fn test_positive_int() {
    let parser = Parser::new("12");
    assert_eq!(Ok(Some(Value::Number(12.0))), parser.parse());
}

#[test]
fn test_positive_decimal() {
    let parser = Parser::new("12.5");
    assert_eq!(Ok(Some(Value::Number(12.5))), parser.parse());
}

#[test]
fn test_negative_int() {
    let parser = Parser::new("-120");
    assert_eq!(Ok(Some(Value::Number(-120.0))), parser.parse());
}

#[test]
fn test_negative_float() {
    let parser = Parser::new("-12.90");
    assert_eq!(Ok(Some(Value::Number(-12.9))), parser.parse());
}

#[test]
fn test_array_empty() {
    let parser = Parser::new("[]");
    assert_eq!(Ok(Some(Value::Array(vec![]))), parser.parse());
}

#[test]
fn test_int_array() {
    let parser = Parser::new("[1, 2, 3]");
    let array = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert_eq!(Ok(Some(array)), parser.parse());
}

#[test]
fn test_array_missing_comma() {
    let parser = Parser::new("[1, 2 3] ");
    assert_eq!(
        Err(ParsingError::TokenAfterValue(Token::Number(3.0))),
        parser.parse()
    );
}

#[test]
fn test_array_missing_close_bracket() {
    let parser = Parser::new("[1, 2, 3 ");
    assert_eq!(
        Err(ParsingError::ReachedEOF(Token::OpenBracket)),
        parser.parse(),
    )
}

#[test]
fn test_array_trailing_comma() {
    let parser = Parser::new("[1 ,2 ,3, ] ");
    assert_eq!(Err(ParsingError::TrailingComma), parser.parse());
}

#[test]
fn test_object_empty() {
    let parser = Parser::new("{}");
    assert_eq!(Ok(Some(Value::Object(HashMap::new()))), parser.parse());
}

#[test]
fn test_object() {
    let parser = Parser::new("{\"one\": 1, \"two\": 2}");
    let mut map = HashMap::new();
    map.insert("one".into(), Value::Number(1.0));
    map.insert("two".into(), Value::Number(2.0));
    let object = Value::Object(map);
    assert_eq!(Ok(Some(object)), parser.parse());
}

#[test]
fn test_object_trailing_comma() {
    let parser = Parser::new(" {\"one\": 1, \"two\": 2, }");
    assert_eq!(Err(ParsingError::TrailingComma), parser.parse());
}

#[test]
fn test_array_mixed() {
    let parser = Parser::new(" [{\"one\": 1, \"two\": 2 } , [1, true, false] ,null ,\"string\"]");

    let mut map = HashMap::new();
    map.insert("one".into(), Value::Number(1.0));
    map.insert("two".into(), Value::Number(2.0));

    let array = Value::Array(vec![
        Value::Object(map),
        Value::Array(vec![
            Value::Number(1.0),
            Value::Bool(true),
            Value::Bool(false),
        ]),
        Value::Null,
        Value::String("string".into()),
    ]);

    assert_eq!(Ok(Some(array)), parser.parse());
}

#[test]
fn test_object_mixed_spaced() {
    let parser = Parser::new(
        " {\"object\": {\"one\": 1, \"two\": 2 } , \"array\": [1, 2] , \"number\": 3 }",
    );

    let mut map_in = HashMap::new();
    map_in.insert("one".into(), Value::Number(1.0));
    map_in.insert("two".into(), Value::Number(2.0));

    let mut map_out = HashMap::new();
    map_out.insert("object".into(), Value::Object(map_in));
    map_out.insert(
        "array".into(),
        Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]),
    );
    map_out.insert("number".into(), Value::Number(3.0));

    let object = Value::Object(map_out);
    assert_eq!(Ok(Some(object)), parser.parse());
}
