use crate::{
    ast::{
        expr::{BinOp, Expr, ExprKind, Literal, LogOp, UnOp},
        stmt::Stmt,
    },
    lexer::{Token, TokenKind},
    parser::ParsingError,
};
use std::iter::Peekable;

pub struct Parser<I: Iterator<Item = Token>> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
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
            Some(TokenKind::Fun) => {
                self.advance().unwrap();
                self.function("function")
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

    fn function(&mut self, kind: &str) -> Result<Stmt, ParsingError> {
        let name = match self.peek_kind() {
            Some(TokenKind::Identifier(id)) => self.advance().unwrap(),
            _ => return Err(ParsingError::GeneralError(format!("Expect {} name.", kind))),
        };

        match self.peek_kind() {
            Some(TokenKind::LeftParen) => {
                self.advance().unwrap();
            }
            _ => {
                return Err(ParsingError::GeneralError(format!(
                    "Expect '(' after {} name",
                    kind
                )))
            }
        }

        let mut params = vec![];

        match self.peek_kind() {
            Some(TokenKind::RightParen) => (),
            _ => loop {
                if params.len() >= 255 {
                    return Err(ParsingError::GeneralError(
                        "Can't have more than 255 parameters.".into(),
                    ));
                }

                match self.peek_kind() {
                    Some(TokenKind::Identifier(_)) => params.push(self.advance().unwrap()),
                    _ => return Err(ParsingError::GeneralError("Expect parameter name.".into())),
                }

                match self.peek_kind() {
                    Some(TokenKind::Comma) => self.advance().unwrap(),
                    _ => break,
                };
            },
        };

        match self.peek_kind() {
            Some(TokenKind::RightParen) => self.advance().unwrap(),
            _ => {
                return Err(ParsingError::GeneralError(
                    "Expect ')' after parameters".into(),
                ))
            }
        };

        match self.peek_kind() {
            Some(TokenKind::LeftBrace) => {
                self.advance().unwrap();
                let body = self.block()?;
                Ok(Stmt::Function(name, params, body))
            }
            _ => Err(ParsingError::GeneralError(format!(
                "Expect '{{' before {} body",
                kind
            ))),
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
            Some(TokenKind::LeftBrace) => {
                self.advance().unwrap();
                Ok(Stmt::Block(self.block()?))
            }
            Some(TokenKind::If) => {
                self.advance().unwrap();
                self.if_statement()
            }
            Some(TokenKind::While) => {
                self.advance().unwrap();
                self.while_statement()
            }
            Some(TokenKind::For) => {
                self.advance().unwrap();
                self.for_statement()
            }
            Some(TokenKind::Return) => {
                let token = self.advance().unwrap();
                self.return_statement(token)
            }
            _ => self.expression_statement(),
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek_kind() {
            Some(TokenKind::LeftParen) => self.advance().unwrap(),
            _ => return Err(ParsingError::GeneralError("Expect '(' after 'if'".into())),
        };

        let condition = self.expression()?;

        match self.peek_kind() {
            Some(TokenKind::RightParen) => self.advance().unwrap(),
            _ => return Err(ParsingError::GeneralError("Expect ')' after 'if'".into())),
        };

        let then_branch = self.statement()?;
        let else_branch = match self.peek_kind() {
            Some(TokenKind::Else) => {
                self.advance().unwrap();
                Some(self.statement()?)
            }
            _ => None,
        };

        Ok(Stmt::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek_kind() {
            Some(TokenKind::LeftParen) => self.advance().unwrap(),
            _ => return Err(ParsingError::GeneralError("Expect '(' after 'if'".into())),
        };

        let condition = self.expression()?;

        match self.peek_kind() {
            Some(TokenKind::RightParen) => self.advance().unwrap(),
            _ => return Err(ParsingError::GeneralError("Expect ')' after 'if'".into())),
        };

        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<Stmt, ParsingError> {
        match self.peek_kind() {
            Some(TokenKind::LeftParen) => self.advance().unwrap(),
            _ => return Err(ParsingError::GeneralError("Expect '(' after 'if'".into())),
        };

        let initializer = match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                self.advance().unwrap();
                None
            }
            Some(TokenKind::Var) => {
                self.advance().unwrap();
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };

        let mut condition = match self.peek_kind() {
            Some(TokenKind::Semicolon) => None,
            _ => Some(self.expression()?),
        };

        match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                self.advance().unwrap();
            }
            _ => {
                return Err(ParsingError::GeneralError(
                    "Expect ';' after loop conditions.".into(),
                ))
            }
        }

        let increment = match self.peek_kind() {
            Some(TokenKind::RightParen) => None,
            _ => Some(self.expression()?),
        };

        match self.peek_kind() {
            Some(TokenKind::RightParen) => {
                self.advance().unwrap();
            }
            _ => {
                return Err(ParsingError::GeneralError(
                    "Expect ')' after for clauses.".into(),
                ))
            }
        }

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        if condition.is_none() {
            condition.replace(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
            });
        }

        body = Stmt::While(condition.unwrap(), Box::new(body));

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
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

    fn return_statement(&mut self, keyword: Token) -> Result<Stmt, ParsingError> {
        let value = match self.peek_kind() {
            Some(TokenKind::Semicolon) => None,
            _ => Some(self.expression()?),
        };

        match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                self.advance().unwrap();
                Ok(Stmt::Return(keyword, value))
            }
            _ => Err(ParsingError::GeneralError(
                "Expect ';' after return value".into(),
            )),
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

    fn block(&mut self) -> Result<Vec<Stmt>, ParsingError> {
        let mut stmts = vec![];

        while !self.is_at_end() {
            match self.peek_kind() {
                Some(TokenKind::RightBrace) => break,
                _ => {
                    if let Some(declaration) = self.declaration() {
                        stmts.push(declaration);
                    }
                }
            }
        }

        if let Some(TokenKind::RightBrace) = self.peek_kind() {
            self.advance().unwrap();
            Ok(stmts)
        } else {
            Err(ParsingError::GeneralError("Expect '}' after block.".into()))
        }
    }

    fn expression(&mut self) -> Result<Expr, ParsingError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParsingError> {
        let expr = self.or()?;

        match self.peek_kind() {
            Some(TokenKind::Eq) => {
                self.advance().unwrap();
                let value = self.assignment()?;
                match expr.kind {
                    ExprKind::Variable(name) => Ok(Expr {
                        kind: ExprKind::Assign(name, Box::new(value)),
                    }),
                    _ => Err(ParsingError::GeneralError(
                        "Invalid assignment target".into(),
                    )),
                }
            }
            _ => Ok(expr),
        }
    }

    fn or(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.and()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::Or) => {
                    let operator_token = self.advance().unwrap();
                    let operator = LogOp::try_from(operator_token.value).unwrap();
                    let right = self.and()?;
                    expr = Expr {
                        kind: ExprKind::Logical(operator, Box::new(expr), Box::new(right)),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.equality()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::And) => {
                    let operator_token = self.advance().unwrap();
                    let operator = LogOp::try_from(operator_token.value).unwrap();
                    let right = self.equality()?;
                    expr = Expr {
                        kind: ExprKind::Logical(operator, Box::new(expr), Box::new(right)),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
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
            _ => self.call(),
        }
    }

    fn call(&mut self) -> Result<Expr, ParsingError> {
        let mut expr = self.primary()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::LeftParen) => {
                    self.advance().unwrap();
                    expr = self.finish_call(expr)?
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParsingError> {
        let mut arguments = vec![];

        match self.peek_kind() {
            Some(TokenKind::RightParen) => (),
            _ => loop {
                if arguments.len() >= 255 {
                    return Err(ParsingError::GeneralError(
                        "Can't have more than 255 arguments".into(),
                    ));
                }

                arguments.push(self.expression()?);
                match self.peek_kind() {
                    Some(TokenKind::Comma) => {
                        self.advance().unwrap();
                    }
                    _ => break,
                }
            },
        }

        match self.peek_kind() {
            Some(TokenKind::RightParen) => {
                self.advance().unwrap();
                Ok(Expr {
                    kind: ExprKind::Call(Box::new(callee), arguments),
                })
            }
            _ => Err(ParsingError::GeneralError(
                "Expect ')' after arguments.".into(),
            )),
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
