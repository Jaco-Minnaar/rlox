use self::cursor::{Cursor, EOF_CHAR};

mod cursor;

pub enum LexingError {
    UnidentifiedToken,
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    Ne,
    Eq,
    EqEq,
    Gt,
    Ge,
    Lt,
    Le,

    Identifier(String),
    String(String),
    Number(f64),

    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Whitespace,
    Unknown,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: TokenKind,
    pub length: usize,
    pub lexeme: String,
}

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);

    std::iter::from_fn(move || {
        if cursor.is_eof() {
            None
        } else {
            cursor.reset_len_consumed();
            Some(cursor.advance_token())
        }
    })
}

impl Cursor<'_> {
    fn advance_token(&mut self) -> Token {
        let c = match self.bump() {
            Some(c) => c,
            None => EOF_CHAR,
        };

        let (token_kind, lexeme) = match c {
            '(' => (TokenKind::LeftParen, c.to_string()),
            ')' => (TokenKind::RightParen, c.to_string()),
            '{' => (TokenKind::LeftBrace, c.to_string()),
            '}' => (TokenKind::RightBrace, c.to_string()),
            ',' => (TokenKind::Comma, c.to_string()),
            '.' => (TokenKind::Dot, c.to_string()),
            '-' => (TokenKind::Minus, c.to_string()),
            '+' => (TokenKind::Plus, c.to_string()),
            ';' => (TokenKind::Semicolon, c.to_string()),
            '*' => (TokenKind::Star, c.to_string()),
            '!' => {
                if self.first() == '=' {
                    let mut lex = String::from(c);
                    let c = self.bump().unwrap();
                    lex.push(c);
                    (TokenKind::Ne, lex)
                } else {
                    (TokenKind::Bang, c.to_string())
                }
            }
            '=' => {
                if self.first() == '=' {
                    let mut lex = String::from(c);
                    let c = self.bump().unwrap();
                    lex.push(c);
                    (TokenKind::EqEq, lex)
                } else {
                    (TokenKind::Eq, c.to_string())
                }
            }
            '>' => {
                if self.first() == '=' {
                    let mut lex = String::from(c);
                    let c = self.bump().unwrap();
                    lex.push(c);
                    (TokenKind::Ge, lex)
                } else {
                    (TokenKind::Gt, c.to_string())
                }
            }
            '<' => {
                if self.first() == '=' {
                    let mut lex = String::from(c);
                    let c = self.bump().unwrap();
                    lex.push(c);
                    (TokenKind::Le, lex)
                } else {
                    (TokenKind::Lt, c.to_string())
                }
            }
            '/' => {
                if self.first() == '/' {
                    self.eat_while(|c| c != '\n');
                    return self.advance_token();
                } else {
                    (TokenKind::Slash, c.to_string())
                }
            }
            // c if c.is_whitespace() => (TokenKind::Whitespace, c.to_string()),
            c if c.is_whitespace() => return self.advance_token(),
            '"' => self.string(),
            c if c.is_digit(10) => self.number(c),
            c if c.is_alphabetic() || c == '_' => self.identifier(c),
            EOF_CHAR => (TokenKind::Eof, c.to_string()),
            _ => (TokenKind::Unknown, c.to_string()),
        };

        Token {
            value: token_kind,
            length: self.len_consumed(),
            lexeme,
        }
    }

    fn string(&mut self) -> (TokenKind, String) {
        let mut val = String::new();
        while let Some(c) = self.bump() {
            if c == '"' {
                let lex = format!("\"{}\"", &val);
                return (TokenKind::String(val), lex);
            }

            val.push(c)
        }

        (TokenKind::Eof, String::new())
    }

    fn number(&mut self, first_digit: char) -> (TokenKind, String) {
        let mut val = String::from(first_digit);
        while self.first().is_digit(10) {
            let c = self.bump().unwrap();
            val.push(c);
        }

        if self.first() == '.' && self.second().is_digit(10) {
            val.push(self.bump().unwrap());
            while self.first().is_digit(10) {
                let c = self.bump().unwrap();
                val.push(c);
            }
        }

        let num: f64 = val.parse().unwrap();

        (TokenKind::Number(num), val)
    }

    fn identifier(&mut self, starting_char: char) -> (TokenKind, String) {
        let mut val = String::from(starting_char);

        while self.first().is_alphanumeric() || self.first() == '_' {
            let c = self.bump().unwrap();
            val.push(c);
        }

        let lexeme = val.clone();
        let token = match val.as_str() {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier(val),
        };

        (token, lexeme)
    }

    fn is_part_number(c: char) -> bool {
        match c {
            c if c.is_digit(10) => true,
            '.' => true,
            _ => false,
        }
    }
}
