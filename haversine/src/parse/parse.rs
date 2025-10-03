use super::{Token, TokenizeError, Tokenizer, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParsingError {
    MissingColon,
    TrailingComma,
    ExtraData,
    InvalidKey(Value),
    ReachedEOF(Token),
    StartingToken(Token),
    Tokenize(TokenizeError),
    TryFromToken(Token),
    TokenAfterValue(Token),
    DuplicateObjectKey(String),
}

impl From<TokenizeError> for ParsingError {
    fn from(error: TokenizeError) -> ParsingError {
        ParsingError::Tokenize(error)
    }
}

pub(crate) struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
        }
    }

    pub(crate) fn parse(mut self) -> Result<Option<Value>, ParsingError> {
        let parsed = self.parse_value()?;
        if self.tokenizer.next_token() != Ok(Token::Eof) {
            return Err(ParsingError::ExtraData);
        }
        Ok(parsed)
    }

    pub(crate) fn parse_value(&mut self) -> Result<Option<Value>, ParsingError> {
        match self.tokenizer.next_token() {
            Ok(Token::Eof) => Ok(None),
            Ok(Token::Null) => Ok(Some(Value::Null)),
            Ok(Token::Bool(b)) => Ok(Some(Value::Bool(b))),
            Ok(Token::String(s)) => Ok(Some(Value::String(s))),
            Ok(Token::Number(n)) => Ok(Some(Value::Number(n))),
            Ok(Token::OpenBracket) => self.parse_array().map(Some),
            Ok(Token::OpenBrace) => self.parse_object().map(Some),
            Ok(t) => Err(ParsingError::StartingToken(t)),
            Err(e) => Err(ParsingError::Tokenize(e)),
        }
    }

    fn parse_array(&mut self) -> Result<Value, ParsingError> {
        let mut items = Vec::new();

        // Handle empty array right away: `[]`
        match self.tokenizer.peek_next()? {
            Token::CloseBracket => {
                let t = self.tokenizer.next_token()?;
                debug_assert_eq!(t, Token::CloseBracket);
                return Ok(Value::Array(items));
            }
            Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBracket)),
            _ => {}
        }

        loop {
            let v = self.parse_value()?.expect("Guaranteed to not be EOF");
            items.push(v);

            // After a value we must see either `,` (more) or `]` (end)
            match self.tokenizer.next_token()? {
                Token::Comma => {
                    // Disallow trailing comma: `,]`
                    match self.tokenizer.peek_next()? {
                        Token::CloseBracket => return Err(ParsingError::TrailingComma),
                        Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBracket)),
                        _ => {}
                    }
                }
                Token::CloseBracket => return Ok(Value::Array(items)),
                Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBracket)),
                tok => return Err(ParsingError::TokenAfterValue(tok)),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Value, ParsingError> {
        let mut map = HashMap::<String, Value>::new();

        // Empty object: `{}`
        match self.tokenizer.peek_next()? {
            Token::CloseBrace => {
                let t = self.tokenizer.next_token()?;
                debug_assert_eq!(t, Token::CloseBrace);
                return Ok(Value::Object(map));
            }
            Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBrace)),
            _ => {}
        }

        loop {
            // Key must be a string
            let key = match self.parse_value()? {
                Some(Value::String(s)) => s,
                Some(val) => return Err(ParsingError::InvalidKey(val)),
                None => return Err(ParsingError::ReachedEOF(Token::OpenBrace)),
            };

            // Colon after key
            match self.tokenizer.next_token()? {
                Token::Colon => {}
                Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBrace)),
                tok => return Err(ParsingError::MissingColon),
            }

            let value = self.parse_value()?.expect("Guaranteed to not be EOF");

            // Forbit duplicate keys
            if map.contains_key(&key) {
                return Err(ParsingError::DuplicateObjectKey(key));
            }
            map.insert(key, value);

            // After a member, require `,` or `}`
            match self.tokenizer.next_token()? {
                Token::Comma => {
                    // Forbid trailing comma: `,}`
                    match self.tokenizer.peek_next()? {
                        Token::CloseBrace => return Err(ParsingError::TrailingComma),
                        Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBrace)),
                        _ => {} // continue parsing next member
                    }
                }
                Token::CloseBrace => return Ok(Value::Object(map)),
                Token::Eof => return Err(ParsingError::ReachedEOF(Token::OpenBrace)),
                tok => return Err(ParsingError::TokenAfterValue(tok)),
            }
        }
    }
}
