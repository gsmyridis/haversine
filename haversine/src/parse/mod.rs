use std::collections::HashMap;
use std::str::Chars;

const EOF: char = '\0';
const STRING_START: char = '"';
const STRING_END: char = '"';
const OPEN_BRACE: char = '{';
const CLOSE_BRACE: char = '}';
const OPEN_BRACKET: char = '[';
const CLOSE_BRACKET: char = ']';
const COMMA: char = ',';
const COLON: char = ':';

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParsingError {
    NoValue,
    InvalidIdent,
    MissingColon,
    TrailingComma,
    InvalidFormat,
    Number(String),
    InvalidKeyType,
    ReachedEOF(&'static str),
    StartingCharacter(String),
}

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

impl Value {
    pub(crate) fn get_number(&self) -> f64 {
        match self {
            Self::Number(num) => *num,
            _ => panic!("Cannot extract number."),
        }
    }
}

/// Parses a file to a JSON object.
/// For a simple introduction to the valid JSON formats
/// visit: https://www.json.org/json-en.html
///
/// Note that this parser is currently not complete. It does not handle:
/// - Numbers in scientific notation
/// - Escaped string
pub(crate) struct Cursor<'a> {
    inner: Chars<'a>,
    prev: Option<char>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            inner: input.chars(),
            prev: None,
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Option<Value>, ParsingError> {
        let value = self.next_value();
        match self.bump() {
            Some(_) => Err(ParsingError::InvalidFormat),
            None => value,
        }
    }

    /// It produces the next value.
    fn next_value(&mut self) -> Result<Option<Value>, ParsingError> {
        self.eat_whitespace();
        let first_char = match self.bump() {
            Some(b) => b,
            None => return Ok(None),
        };

        match first_char {
            'n' => self.next_null().map(Some),
            't' => self.next_true().map(|b| Some(Value::Bool(b))),
            'f' => self.next_false().map(|b| Some(Value::Bool(b))),
            STRING_START => self.next_string().map(|string| Some(Value::String(string))),
            OPEN_BRACKET => self.next_array().map(|array| Some(Value::Array(array))),
            OPEN_BRACE => self.next_object().map(|obj| Some(Value::Object(obj))),
            '0'..='9' | '-' => self
                .next_number(first_char)
                .map(|num| Some(Value::Number(num))),
            _ => Err(ParsingError::StartingCharacter(first_char.into())),
        }
    }

    /// Gets the next null value.
    fn next_null(&mut self) -> Result<Value, ParsingError> {
        debug_assert!(matches!(self.prev, None | Some('n')));
        let (second, third, fourth) = (self.bump(), self.bump(), self.bump());
        if second != Some('u') || third != Some('l') || fourth != Some('l') {
            return Err(ParsingError::InvalidIdent);
        }
        Ok(Value::Null)
    }

    fn next_true(&mut self) -> Result<bool, ParsingError> {
        debug_assert!(matches!(self.prev, None | Some('t')));
        let (second, third, fourth) = (self.bump(), self.bump(), self.bump());
        if second != Some('r') || third != Some('u') || fourth != Some('e') {
            return Err(ParsingError::InvalidIdent);
        }
        Ok(true)
    }

    fn next_false(&mut self) -> Result<bool, ParsingError> {
        debug_assert!(matches!(self.prev, None | Some('f')));
        let (second, third, fourth, fifth) = (self.bump(), self.bump(), self.bump(), self.bump());
        if second != Some('a') || third != Some('l') || fourth != Some('s') || fifth != Some('e') {
            return Err(ParsingError::InvalidIdent);
        }
        Ok(false)
    }

    /// Gives the next string.
    /// It takes the next character until it meets `"`.
    fn next_string(&mut self) -> Result<String, ParsingError> {
        debug_assert!(matches!(self.prev, None | Some(STRING_START)));
        let mut string = String::new();
        while let Some(c) = self.bump() {
            if c == STRING_END {
                return Ok(string);
            }
            string.push(c);
        }
        Err(ParsingError::ReachedEOF("\""))
    }

    fn next_number(&mut self, first_digit: char) -> Result<f64, ParsingError> {
        let mut string = format!("{first_digit}");
        loop {
            let next_char = self.peek_next();
            if is_whitespace(next_char)
                || is_closing_punct(next_char)
                || matches!(next_char, COMMA | EOF)
            {
                return string.parse().map_err(|_| ParsingError::Number(string));
            }
            let _ = self.inner.next();
            string.push(next_char);
        }
    }

    fn next_array(&mut self) -> Result<Vec<Value>, ParsingError> {
        debug_assert!(matches!(self.prev, None | Some('[')));

        let mut array = Vec::new();
        loop {
            self.eat_whitespace();
            let next_char = self.peek_next();
            if next_char == CLOSE_BRACKET {
                let _ = self.bump(); // consume the closing bracket
                return Ok(array);
            } else if next_char == EOF {
                return Err(ParsingError::ReachedEOF("]"));
            };

            let value = self.next_value()?.ok_or(ParsingError::NoValue)?;
            array.push(value);
            self.eat_whitespace();
            if self.peek_next() == COMMA {
                let _ = self.bump();
                self.eat_whitespace();
                if self.peek_next() == CLOSE_BRACKET {
                    return Err(ParsingError::TrailingComma);
                }
            }
        }
    }

    fn next_object(&mut self) -> Result<HashMap<String, Value>, ParsingError> {
        debug_assert!(matches!(self.prev, None | Some('{')));
        self.eat_whitespace();

        let mut map = HashMap::new();
        loop {
            let next_char = self.peek_next();
            if next_char == CLOSE_BRACE {
                let _ = self.bump();
                return Ok(map);
            } else if next_char == EOF {
                return Err(ParsingError::ReachedEOF("}"));
            }

            let key = match self.next_value()?.ok_or(ParsingError::NoValue)? {
                Value::String(string) => string,
                _ => return Err(ParsingError::InvalidKeyType),
            };

            self.eat_whitespace(); // Eat the whitespace after `"`
            if self.bump() != Some(COLON) {
                return Err(ParsingError::MissingColon);
            }
            self.eat_whitespace(); // Eat the whitespace after `:`

            let val = self.next_value()?.ok_or(ParsingError::NoValue)?;

            map.insert(key, val);
            self.eat_whitespace();
            if self.peek_next() == COMMA {
                let _ = self.bump();
                self.eat_whitespace();
                if self.peek_next() == CLOSE_BRACE {
                    return Err(ParsingError::TrailingComma);
                }
            }
        }
    }

    /// Eats the whitespace.
    fn eat_whitespace(&mut self) {
        self.eat_while(is_whitespace);
    }

    /// Checks if the 'Cursor' has reached the end of file.
    fn is_eof(&self) -> bool {
        self.peek_next() == EOF
    }

    /// Bumps the 'Cursor' returning the next byte in the file.
    fn bump(&mut self) -> Option<char> {
        let next = self.inner.next();
        self.prev = next;
        next
    }

    /// Peeks the next byte in the 'Cursor'.
    fn peek_next(&self) -> char {
        self.inner.clone().next().unwrap_or(EOF)
    }

    /// Eats the next byte while the predicate is true of the 'Cursor' has
    /// reached the end of file.
    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek_next()) && !self.is_eof() {
            let _ = self.inner.next();
        }
    }
}

/// Checks if character 'c' is a whitespace.
fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// Checks if the character 'c' is closing punctuation, ']' or '}'.
fn is_closing_punct(c: char) -> bool {
    matches!(c, CLOSE_BRACE | CLOSE_BRACKET)
}

#[cfg(test)]
mod test {
    use super::{Cursor, ParsingError, Value};
    use std::collections::HashMap;

    #[test]
    fn test_null() {
        let mut cursor = Cursor::new("null");
        assert_eq!(Ok(Some(Value::Null)), cursor.next_value());
    }

    #[test]
    fn test_null_spaced() {
        let mut cursor = Cursor::new(" null ");
        assert_eq!(Ok(Some(Value::Null)), cursor.next_value());
    }

    #[test]
    fn test_true() {
        let mut cursor = Cursor::new("true");
        assert_eq!(Ok(Some(Value::Bool(true))), cursor.next_value());
    }

    #[test]
    fn test_true_spaced() {
        let mut cursor = Cursor::new(" true ");
        assert_eq!(Ok(Some(Value::Bool(true))), cursor.next_value());
    }

    #[test]
    fn test_false() {
        let mut cursor = Cursor::new("false");
        assert_eq!(Ok(Some(Value::Bool(false))), cursor.next_value());
    }

    #[test]
    fn test_false_spaced() {
        let mut cursor = Cursor::new(" false ");
        assert_eq!(Ok(Some(Value::Bool(false))), cursor.next_value());
    }

    #[test]
    fn test_string() {
        let mut cursor = Cursor::new("\"Hello, this is a string!\"");
        let string = Value::String("Hello, this is a string!".into());
        assert_eq!(Ok(Some(string)), cursor.next_value());
    }

    #[test]
    fn test_string_spaced() {
        let mut cursor = Cursor::new(" \"Hello, this is a string!\" ");
        let string = Value::String("Hello, this is a string!".into());
        assert_eq!(Ok(Some(string)), cursor.next_value());
    }

    #[test]
    fn test_positive_int() {
        let mut cursor = Cursor::new("12");
        assert_eq!(Ok(Some(Value::Number(12.0))), cursor.next_value());
    }

    #[test]
    fn test_positive_int_spaced() {
        let mut cursor = Cursor::new(" 12 ");
        assert_eq!(Ok(Some(Value::Number(12.0))), cursor.next_value());
    }

    #[test]
    fn test_positive_decimal() {
        let mut cursor = Cursor::new("12.5");
        assert_eq!(Ok(Some(Value::Number(12.5))), cursor.next_value());
    }

    #[test]
    fn test_positive_decimal_spaced() {
        let mut cursor = Cursor::new(" 12.5 ");
        assert_eq!(Ok(Some(Value::Number(12.5))), cursor.next_value());
    }

    #[test]
    fn test_negative_int() {
        let mut cursor = Cursor::new("-120");
        assert_eq!(Ok(Some(Value::Number(-120.0))), cursor.next_value());
    }

    #[test]
    fn test_negative_int_spaced() {
        let mut cursor = Cursor::new(" -120 ");
        assert_eq!(Ok(Some(Value::Number(-120.0))), cursor.next_value());
    }

    #[test]
    fn test_negative_float() {
        let mut cursor = Cursor::new("-12.90");
        assert_eq!(Ok(Some(Value::Number(-12.9))), cursor.next_value());
    }

    #[test]
    fn test_negative_float_spaced() {
        let mut cursor = Cursor::new(" -12.90 ");
        assert_eq!(Ok(Some(Value::Number(-12.9))), cursor.next_value());
    }

    #[test]
    fn test_array_empty() {
        let mut cursor = Cursor::new("[]");
        assert_eq!(Ok(Some(Value::Array(vec![]))), cursor.next_value());
    }

    #[test]
    fn test_array_empty_spaced() {
        let mut cursor = Cursor::new(" [ ] ");
        assert_eq!(Ok(Some(Value::Array(vec![]))), cursor.next_value());
    }

    #[test]
    fn test_int_array() {
        let mut cursor = Cursor::new("[1, 2, 3]");
        let array = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(Ok(Some(array)), cursor.next_value());
    }

    #[test]
    fn test_int_array_spaced() {
        let mut cursor = Cursor::new(" [1 , 2 , 3 ] ");
        let array = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(Ok(Some(array)), cursor.next_value());
    }

    #[test]
    fn test_int_array_trailing_comma() {
        let mut cursor = Cursor::new("[1 ,2 ,3 , ] ");
        assert_eq!(Err(ParsingError::TrailingComma), cursor.next_value());
    }

    #[test]
    fn test_object_empty() {
        let mut cursor = Cursor::new("{}");
        assert_eq!(Ok(Some(Value::Object(HashMap::new()))), cursor.next_value());
    }

    #[test]
    fn test_object_empty_empty() {
        let mut cursor = Cursor::new(" { } ");
        assert_eq!(Ok(Some(Value::Object(HashMap::new()))), cursor.next_value());
    }

    #[test]
    fn test_object() {
        let mut cursor = Cursor::new("{ \"one\": 1, \"two\": 2}");
        let mut map = HashMap::new();
        map.insert("one".into(), Value::Number(1.0));
        map.insert("two".into(), Value::Number(2.0));
        let object = Value::Object(map);
        assert_eq!(Ok(Some(object)), cursor.next_value());
    }

    #[test]
    fn test_object_spaced() {
        let mut cursor = Cursor::new(" { \"one\" : 1, \"two\" : 2 }");
        let mut map = HashMap::new();
        map.insert("one".into(), Value::Number(1.0));
        map.insert("two".into(), Value::Number(2.0));
        let object = Value::Object(map);
        assert_eq!(Ok(Some(object)), cursor.next_value());
    }

    #[test]
    fn test_object_trailing_comma() {
        let mut cursor = Cursor::new(" {\"one\": 1, \"two\": 2, }");
        assert_eq!(Err(ParsingError::TrailingComma), cursor.next_value());
    }

    #[test]
    fn test_array_mixed_spaced() {
        let mut cursor = Cursor::new(
            " [ { \"one\" : 1, \"two\" : 2 } , [ 1 , true , false ] , null , \"string\" ]",
        );

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

        assert_eq!(Ok(Some(array)), cursor.next_value());
    }

    #[test]
    fn test_object_mixed_spaced() {
        let mut cursor = Cursor::new(
            " { \"object\" : { \"one\" : 1, \"two\" : 2 } , \"array\" : [1 , 2 ] , \"number\" : 3 }",
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
        assert_eq!(Ok(Some(object)), cursor.next_value());
    }
}
