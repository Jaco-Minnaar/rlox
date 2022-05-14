use crate::{
    ast::{
        expr::{BinOp, Expr, ExprKind, Literal, UnOp},
        stmt::Stmt,
    },
    lexer::{Token, TokenKind},
    parser::ParsingError,
};
use std::iter::Peekable;

pub struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
    tokens_parsed: usize,
    current_token: Token,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            tokens_parsed: 0,
            current_token: Token {
                value: TokenKind::Unknown,
                length: 0,
                lexeme: "".into(),
            },
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];

        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        statements
    }

    fn peek_kind(&mut self) -> Option<&TokenKind> {
        if let Some(t) = self.tokens.peek() {
            Some(&t.value)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<Token> {
        loop {
            let token = self.tokens.next();

            match token {
                Some(t) => match t.value {
                    TokenKind::Whitespace => continue,
                    _ => return Some(t),
                },
                _ => return None,
            };
        }
    }

    fn is_at_end(&mut self) -> bool {
        match self.peek_kind() {
            Some(TokenKind::Eof) | None => true,
            _ => false,
        }
    }

    fn sync(&mut self) {
        loop {
            let token = match self.advance() {
                Some(t) => t,
                None => return,
            };

            if let TokenKind::Semicolon = token.value {
                return;
            }

            match self.peek_kind() {
                Some(t) => match t {
                    TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return => return,
                    _ => (),
                },
                _ => return,
            }
        }
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let result = match self.peek_kind() {
            Some(TokenKind::Var) => {
                self.advance().unwrap();
                self.var_declaration()
            }
            _ => self.statement(),
        };

        match result {
            Ok(v) => Some(v),
            Err(ParsingError::GeneralError(e)) => {
                eprintln!("Parser Error: {}", e);
                self.sync();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek_kind() {
            Some(TokenKind::Identifier(_)) => {
                let name = self.advance().unwrap();

                let initializer = if let Some(TokenKind::Eq) = self.peek_kind() {
                    self.advance().unwrap();
                    Some(self.expression()?)
                } else {
                    None
                };

                match self.peek_kind() {
                    Some(TokenKind::Semicolon) => {
                        self.advance().unwrap();
                        Ok(Stmt::Var(name, initializer))
                    }
                    _ => Err(ParsingError::GeneralError(
                        "Expect ';' after variable declaration.".into(),
                    )),
                }
            }
            _ => Err(ParsingError::GeneralError("Expect variable name.".into())),
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek_kind() {
            Some(TokenKind::Print) => {
                self.advance().unwrap();
                self.print_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, ParsingError> {
        let value = self.expression()?;

        match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                self.advance().unwrap();
                Ok(Stmt::Print(value))
            }
            _ => Err(ParsingError::GeneralError("Expect ';' after value".into())),
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParsingError> {
        let expr = self.expression()?;

        match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                self.advance().unwrap();
                Ok(Stmt::Expression(expr))
            }
            _ => Err(ParsingError::GeneralError("Expect ';' after value".into())),
        }
    }

    fn expression(&mut self) -> Result<Expr, ParsingError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.comparison()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::Ne) | Some(TokenKind::EqEq) => {
                    let operator_token = self.advance().unwrap();
                    let bin_op = BinOp::try_from(operator_token.value).unwrap();
                    let right = self.comparison()?;
                    expr = Expr {
                        kind: ExprKind::Binary(bin_op, Box::new(expr), Box::new(right)),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.term()?;

        loop {
            match self.peek_kind() {
                Some(t) => match t {
                    TokenKind::Gt | TokenKind::Ge | TokenKind::Lt | TokenKind::Le => {
                        let operator_token = self.advance().unwrap();
                        let bin_op = BinOp::try_from(operator_token.value).unwrap();
                        let right = self.term()?;

                        expr = Expr {
                            kind: ExprKind::Binary(bin_op, Box::new(expr), Box::new(right)),
                        }
                    }
                    _ => break,
                },
                _ => break,
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.factor()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::Minus) | Some(TokenKind::Plus) => {
                    let operator_token = self.advance().unwrap();
                    let bin_op = BinOp::try_from(operator_token.value).unwrap();

                    let right = self.factor()?;
                    expr = Expr {
                        kind: ExprKind::Binary(bin_op, Box::new(expr), Box::new(right)),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.unary()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::Slash) | Some(TokenKind::Star) => {
                    let operator_token = self.advance().unwrap();
                    let bin_op = BinOp::try_from(operator_token.value).unwrap();

                    let right = self.unary()?;
                    expr = Expr {
                        kind: ExprKind::Binary(bin_op, Box::new(expr), Box::new(right)),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParsingError> {
        match self.peek_kind() {
            Some(TokenKind::Bang) | Some(TokenKind::Minus) => {
                let operator_token = self.advance().unwrap();
                let un_op = UnOp::try_from(operator_token.value).unwrap();
                let right = self.unary()?;
                Ok(Expr {
                    kind: ExprKind::Unary(un_op, Box::new(right)),
                })
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, ParsingError> {
        let token = match self.advance() {
            Some(token) => token,
            None => return Err(ParsingError::GeneralError("Unexpected EOF".to_string())),
        };

        let expr = match token.value {
            TokenKind::False => Expr {
                kind: ExprKind::Literal(Literal::Bool(false)),
            },
            TokenKind::True => Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
            },
            TokenKind::Nil => Expr {
                kind: ExprKind::Literal(Literal::Nil),
            },
            TokenKind::Number(num) => Expr {
                kind: ExprKind::Literal(Literal::Number(num)),
            },
            TokenKind::String(s) => Expr {
                kind: ExprKind::Literal(Literal::String(s.to_string())),
            },
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                match self.peek_kind() {
                    Some(TokenKind::RightParen) => {
                        self.advance().unwrap();
                        Expr {
                            kind: ExprKind::Grouping(Box::new(expr)),
                        }
                    }
                    _ => {
                        return Err(ParsingError::GeneralError(
                            "Expected ')' after expression".to_string(),
                        ))
                    }
                }
            }
            TokenKind::Identifier(_) => Expr {
                kind: ExprKind::Variable(token),
            },
            _ => {
                return Err(ParsingError::GeneralError(format!(
                    "Unexpected token {:?}",
                    token
                )))
            }
        };

        Ok(expr)
    }
}
