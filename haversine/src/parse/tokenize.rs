use std::str::Chars;

const EOF_CHAR: char = '\0';

/// Token for JSON parser
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    /// End of file
    Eof,
    /// Quote `"`
    Quote,
    /// Open brace `{`
    OpenBrace,
    /// Close brace `}`
    CloseBrace,
    /// Open bracket `[`
    OpenBracket,
    /// Close bracket `[`
    CloseBracket,
    /// Comma `,`
    Comma,
    /// Colon `:`
    Colon,
    /// Null `null`
    Null,
    /// Boolean `true` or `false`
    Bool(bool),
    /// String
    String(String),
    /// Number
    Number(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenizeError {
    InvalidNull,
    InvalidTrue,
    InvalidFalse,
    InvalidNumber(String),
    ReachedEOF(&'static str),
    UnexpectedChar(char),
}

/// visit: https://www.json.org/json-en.html
pub(crate) struct Tokenizer<'a> {
    inner: Chars<'a>,
    prev_char: Option<char>,
}

impl<'a> Tokenizer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            inner: input.chars(),
            prev_char: None,
        }
    }

    pub(crate) fn peek_next(&mut self) -> Result<Token, TokenizeError> {
        let chars = self.inner.clone();
        let token = self.next_token();
        self.inner = chars;
        token
    }

    pub(crate) fn next_token(&mut self) -> Result<Token, TokenizeError> {
        self.eat_whitespace();
        match self.bump() {
            None => Ok(Token::Eof),
            Some('[') => Ok(Token::OpenBracket),
            Some(']') => Ok(Token::CloseBracket),
            Some('{') => Ok(Token::OpenBrace),
            Some('}') => Ok(Token::CloseBrace),
            Some(',') => Ok(Token::Comma),
            Some(':') => Ok(Token::Colon),
            Some('n') => self.next_null(),
            Some('t') => self.next_true(),
            Some('f') => self.next_false(),
            Some('\"') => self.next_string(),
            Some(c) if matches!(c, '0'..='9' | '-') => self.next_number(c),
            Some(c) => Err(TokenizeError::UnexpectedChar(c)),
        }
    }

    fn next_null(&mut self) -> Result<Token, TokenizeError> {
        debug_assert!(matches!(self.prev_char, None | Some('n')));
        let (second, third, fourth) = (self.bump(), self.bump(), self.bump());
        if second != Some('u') || third != Some('l') || fourth != Some('l') {
            return Err(TokenizeError::InvalidNull);
        }
        Ok(Token::Null)
    }

    fn next_true(&mut self) -> Result<Token, TokenizeError> {
        debug_assert!(matches!(self.prev_char, None | Some('t')));
        let (second, third, fourth) = (self.bump(), self.bump(), self.bump());
        if second != Some('r') || third != Some('u') || fourth != Some('e') {
            return Err(TokenizeError::InvalidTrue);
        }
        Ok(Token::Bool(true))
    }

    fn next_false(&mut self) -> Result<Token, TokenizeError> {
        debug_assert!(matches!(self.prev_char, None | Some('f')));
        let (second, third, fourth, fifth) = (self.bump(), self.bump(), self.bump(), self.bump());
        if second != Some('a') || third != Some('l') || fourth != Some('s') || fifth != Some('e') {
            return Err(TokenizeError::InvalidFalse);
        }
        Ok(Token::Bool(false))
    }

    fn next_string(&mut self) -> Result<Token, TokenizeError> {
        debug_assert!(matches!(self.prev_char, None | Some('\"')));
        let mut string = String::new();
        while let Some(c) = self.bump() {
            if c == '\"' {
                return Ok(Token::String(string));
            }
            string.push(c);
        }
        Err(TokenizeError::ReachedEOF("\""))
    }

    fn next_number(&mut self, first_digit: char) -> Result<Token, TokenizeError> {
        // TODO: Debug assert the previous digit
        let mut string = format!("{first_digit}");
        loop {
            let next_char = self.peek_next_char();
            if is_whitespace(next_char) || matches!(next_char, ',' | ']' | '}' | EOF_CHAR) {
                let num = string
                    .parse()
                    .map_err(|_| TokenizeError::InvalidNumber(string))?;
                return Ok(Token::Number(num));
            }
            let _ = self.inner.next();
            string.push(next_char);
        }
    }

    /// Eats the whitespace.
    fn eat_whitespace(&mut self) {
        self.eat_while(is_whitespace);
    }

    /// Checks if the 'Cursor' has reached the end of file.
    fn is_eof(&self) -> bool {
        self.peek_next_char() == EOF_CHAR
    }

    /// Bumps the 'Cursor' returning the next byte in the file.
    fn bump(&mut self) -> Option<char> {
        let next = self.inner.next();
        self.prev_char = next;
        next
    }

    /// Peeks the next byte in the 'Cursor'.
    fn peek_next_char(&self) -> char {
        self.inner.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Eats the next byte while the predicate is true of the 'Cursor' has
    /// reached the end of file.
    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek_next_char()) && !self.is_eof() {
            let _ = self.inner.next();
        }
    }
}

/// Checks if character 'c' is a whitespace.
fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}
