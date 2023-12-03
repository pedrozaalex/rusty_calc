use std::fmt::Display;

use anyhow::Result;
use thiserror::Error;

static SYMBOLS: [char; 10] = [
    /* --- Operators --- */
    '+', '-', '*', '/', '!',
    /* --- Parentheses --- */
    '(', ')',
    /* --- Commands --- */
    '=', // Assign
    ';', // Print
    'q' // Quit
];

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Symbol(char),
    Let,
    Name(String),
    EndStatement,
    Quit,
    Noop,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Symbol(s) => write!(f, "Symbol({})", s),
            Token::Let => write!(f, "Let"),
            Token::Name(n) => write!(f, "Name({})", n),
            Token::EndStatement => write!(f, "EndStatement"),
            Token::Quit => write!(f, "Quit"),
            Token::Noop => write!(f, "Noop"),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum TokenizationError {
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(char),
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
}

type MaybeToken = Option<Token>;

pub struct TokenStream {
    buffer: Vec<char>,
    pos: usize,
    put_back: Vec<Token>,
}

impl TokenStream {
    pub fn new(input: &[u8]) -> TokenStream {
        TokenStream {
            buffer: String::from_utf8_lossy(input).chars().collect(),
            pos: 0,
            put_back: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Result<MaybeToken> {
        if let Some(token) = self.put_back.pop() {
            return Ok(Some(token));
        }

        // Skip whitespaces
        while self.pos < self.buffer.len() && self.buffer[self.pos].is_whitespace() {
            self.pos += 1;
        }

        if self.pos >= self.buffer.len() {
            return Ok(None);
        }

        let c = self.read_char();
        if is_beginning_of_literal(c) {
            self.pos -= 1;
            let number = self.read_number()?;
            Ok(Some(Token::Number(number)))
        } else if is_valid_symbol(c) {
            match c {
                ';' => if self.pos < self.buffer.len() - 1 { Ok(Some(Token::EndStatement)) } else { Ok(None) },
                'q' => Ok(Some(Token::Quit)),
                _ => Ok(Some(Token::Symbol(c)))
            }
        } else if c.is_alphabetic() {
            self.pos -= 1;
            let string = self.read_string();

            if string == "let" {
                return Ok(Some(Token::Let));
            }

            Ok(Some(Token::Name(string)))
        } else {
            Err(TokenizationError::InvalidSymbol(c).into())
        }
    }

    pub fn peek(&mut self) -> Result<MaybeToken> {
        let token = self.next()?;
        if let Some(ref token) = token {
            self.put_back(token.clone());
        }
        Ok(token)
    }

    pub fn put_back(&mut self, token: Token) {
        self.put_back.push(token);
    }

    // The current expression is deemed invalid, discard everything until the next semicolon, or the end of the input
    pub fn discard_invalid(&mut self) {
        while self.pos <= self.buffer.len() {
            if self.pos == self.buffer.len() || self.buffer[self.pos] == ';' {
                break;
            }

            self.pos += 1;
        }
    }

    fn read_number(&mut self) -> Result<f64> {
        let mut number = String::new();
        while self.pos < self.buffer.len() {
            let c = self.buffer[self.pos];
            if is_part_of_literal(c, &number) {
                number.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        number.parse().map_err(|_| TokenizationError::InvalidNumber(number).into())
    }

    fn read_char(&mut self) -> char {
        let c = self.buffer[self.pos];
        self.pos += 1;
        c
    }

    fn read_string(&mut self) -> String {
        let mut name = String::new();
        while self.pos < self.buffer.len() {
            let c = self.buffer[self.pos];
            if c.is_alphanumeric() {
                name.push(c);
                self.pos += 1;
            } else { break; }
        }
        name
    }
}

impl Iterator for TokenStream {
    type Item = Result<MaybeToken>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

fn is_beginning_of_literal(c: char) -> bool {
    c.is_digit(10) || c == '.'
}

fn is_part_of_literal(c: char, ctx: &str) -> bool {
    // account for scientific notation
    if ctx.ends_with('e') || ctx.ends_with('E') {
        return c.is_digit(10) || c == '-' || c == '+';
    }

    c.is_digit(10) || c == '.' || c == 'e' || c == 'E'
}


fn is_valid_symbol(c: char) -> bool {
    SYMBOLS.contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        input: &'static str,
    }

    impl TestCase {
        fn input(input: &'static str) -> TestCase {
            TestCase {
                input,
            }
        }

        fn expect(self, expected: Vec<Token>) {
            let mut ts = TokenStream::new(self.input.as_bytes());
            let mut actual = Vec::new();
            while let Some(token) = ts.next().unwrap() {
                actual.push(token);
            }
            assert_eq!(actual, expected);
        }

        fn expect_err(self, expected: TokenizationError) {
            let mut ts = TokenStream::new(self.input.as_bytes());
            assert_eq!(ts.next().unwrap_err().downcast::<TokenizationError>().unwrap(), expected);
        }
    }


    #[test]
    fn test_next_with_number() {
        TestCase::input("123")
            .expect(vec![Token::Number(123.0)]);
    }

    #[test]
    fn test_next_with_symbol() {
        TestCase::input("+")
            .expect(vec![Token::Symbol('+')]);
    }

    #[test]
    fn test_next_with_empty_string() {
        TestCase::input("")
            .expect(vec![]);
    }

    #[test]
    fn test_next_with_whitespace() {
        TestCase::input("    ")
            .expect(vec![]);
    }

    #[test]
    fn test_next_with_multiple_numbers() {
        TestCase::input("123 456 789")
            .expect(vec![
                Token::Number(123.0),
                Token::Number(456.0),
                Token::Number(789.0),
            ]);
    }

    #[test]
    fn test_next_with_multiple_symbols() {
        TestCase::input("+ - * /")
            .expect(vec![
                Token::Symbol('+'),
                Token::Symbol('-'),
                Token::Symbol('*'),
                Token::Symbol('/'),
            ]);
    }

    #[test]
    fn test_next_with_mixed_numbers_and_symbols() {
        TestCase::input("123 + 456 -789")
            .expect(vec![
                Token::Number(123.0),
                Token::Symbol('+'),
                Token::Number(456.0),
                Token::Symbol('-'),
                Token::Number(789.0),
            ]);
    }

    #[test]
    fn test_next_with_decimal_number() {
        TestCase::input("123.456").expect(vec![Token::Number(123.456)]);
    }

    #[test]
    fn test_next_with_scientific_notation() {
        TestCase::input("1.23e-4").expect(vec![Token::Number(1.23e-4)]);
    }

    #[test]
    fn test_next_with_invalid_symbol() {
        TestCase::input("@").expect_err(TokenizationError::InvalidSymbol('@'));
    }

    #[test]
    fn test_next_with_leading_whitespace() {
        TestCase::input("  123").expect(vec![Token::Number(123.0)]);
    }

    #[test]
    fn test_next_with_trailing_whitespace() {
        TestCase::input("123  ").expect(vec![Token::Number(123.0)]);
    }

    #[test]
    fn test_next_with_whitespace_between_numbers() {
        TestCase::input("123   456").expect(vec![Token::Number(123.0), Token::Number(456.0)]);
    }

    #[test]
    fn test_next_with_whitespace_between_symbols() {
        TestCase::input("+   -").expect(vec![Token::Symbol('+'), Token::Symbol('-')]);
    }

    #[test]
    fn test_next_with_mixed_numbers_symbols_and_whitespace() {
        TestCase::input("123   +   456   -   789").expect(vec![
            Token::Number(123.0),
            Token::Symbol('+'),
            Token::Number(456.0),
            Token::Symbol('-'),
            Token::Number(789.0),
        ]);
    }

    #[test]
    fn test_next_subtraction() {
        TestCase::input("123-456").expect(vec![
            Token::Number(123.0),
            Token::Symbol('-'),
            Token::Number(456.0),
        ]);
    }

    #[test]
    fn test_next_with_parentheses() {
        TestCase::input("(123 + 456)").expect(vec![
            Token::Symbol('('),
            Token::Number(123.0),
            Token::Symbol('+'),
            Token::Number(456.0),
            Token::Symbol(')'),
        ]);
    }

    #[test]
    fn test_next_with_nested_parentheses() {
        TestCase::input("(123 + (456 - 789))").expect(vec![
            Token::Symbol('('),
            Token::Number(123.0),
            Token::Symbol('+'),
            Token::Symbol('('),
            Token::Number(456.0),
            Token::Symbol('-'),
            Token::Number(789.0),
            Token::Symbol(')'),
            Token::Symbol(')'),
        ]);
    }

    #[test]
    fn test_next_with_invalid_number() {
        TestCase::input("123.456.789").expect_err(TokenizationError::InvalidNumber("123.456.789".to_string()));
    }

    #[test]
    fn test_next_with_unbalanced_parentheses() {
        TestCase::input("(123 + 456").expect(vec![
            Token::Symbol('('),
            Token::Number(123.0),
            Token::Symbol('+'),
            Token::Number(456.0),
        ]);
    }

    #[test]
    fn test_next_with_nested_unbalanced_parentheses() {
        TestCase::input("(123 + (456 - 789)").expect(vec![
            Token::Symbol('('),
            Token::Number(123.0),
            Token::Symbol('+'),
            Token::Symbol('('),
            Token::Number(456.0),
            Token::Symbol('-'),
            Token::Number(789.0),
            Token::Symbol(')'),
        ]);
    }

    #[test]
    fn test_next_with_unexpected_symbol() {
        TestCase::input("123 + * 456").expect(vec![
            Token::Number(123.0),
            Token::Symbol('+'),
            Token::Symbol('*'),
            Token::Number(456.0),
        ]);
    }
}
