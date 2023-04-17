use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::lexer::token::{Literal, Token, TokenType};
use crate::Lox;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
struct ParseError;

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError")
    }
}

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,

    interpreter: &'a mut Lox,
}

impl Parser<'_> {
    pub fn new(interpreter: &mut Lox, tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            interpreter,
        }
    }

    pub fn parse(&mut self) -> Vec<Option<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_end() {
            statements.push(self.declaration())
        }

        statements
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let exp = self.or()?;

        if self.match_tokens(&vec![TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match exp {
                Expr::Variable(name) => {
                    return Ok(Expr::Assign(name, Box::new(value)));
                }
                _ => {
                    self.error(&equals, "Invalid assignment target.");
                }
            }
        }

        Ok(exp)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.match_tokens(&vec![TokenType::Fun]) {
            match self.function("function") {
                Ok(s) => return Some(s),
                Err(_) => {
                    // eprintln!("{err}");
                    self.synchronize();
                    return None;
                }
            }
        }

        if self.match_tokens(&vec![TokenType::Var]) {
            match self.var_declaration() {
                Ok(s) => return Some(s),
                Err(_) => {
                    // eprintln!("{err}");
                    self.synchronize();
                    return None;
                }
            }
        }

        match self.statement() {
            Ok(s) => Some(s),
            Err(_) => {
                // eprintln!("{err}");
                self.synchronize();
                None
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_tokens(&vec![TokenType::For]) {
            return self.for_stmt();
        }

        if self.match_tokens(&vec![TokenType::If]) {
            return self.if_stmt();
        }

        if self.match_tokens(&vec![TokenType::Print]) {
            return self.print_stmt();
        }

        if self.match_tokens(&vec![TokenType::While]) {
            return self.while_stmt();
        }

        if self.match_tokens(&vec![TokenType::LeftBrace]) {
            return self.block();
        }

        self.expression_stmt()
    }

    fn for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_tokens(&vec![TokenType::Semicolon]) {
            None
        } else if self.match_tokens(&vec![TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_stmt()?)
        };

        let mut condition: Option<Expr> = None;
        if !self.check(&TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let mut increment: Option<Expr> = None;
        if !self.check(&TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(exp) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(exp)]);
        }

        body = match condition {
            Some(cond) => Stmt::While(cond, Box::new(body)),
            None => {
                let cond = Expr::LiteralExpr(Literal::Bool(true));
                Stmt::While(cond, Box::new(body))
            }
        };

        if let Some(init_stmt) = initializer {
            body = Stmt::Block(vec![init_stmt, body]);
        }

        Ok(body)
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_tokens(&vec![TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        match self.consume(TokenType::Semicolon, "Expect ';' after value.") {
            Ok(_) => (),
            Err(err) => return Err(err),
        };
        Ok(Stmt::Print(value))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let mut initializer = Expr::LiteralExpr(Literal::None);
        if self.match_tokens(&vec![TokenType::Equal]) {
            initializer = self.expression()?;
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var(name, initializer))
    }

    fn while_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn expression_stmt(&mut self) -> Result<Stmt, ParseError> {
        let exp = self.expression()?;
        match self.consume(TokenType::Semicolon, "Expect ';' after value.") {
            Ok(_) => (),
            Err(err) => return Err(err),
        };
        Ok(Stmt::Expression(exp))
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, ParseError> {
        let mut message = String::new();
        message.push_str("Expect ");
        message.push_str(kind);
        message.push_str(" name.");
        let name = self.consume(TokenType::Identifier, &message)?;

        message = String::new();
        message.push_str("Expect '(' after ");
        message.push_str(kind);
        message.push_str(" name.");
        self.consume(TokenType::LeftParen, &message)?;

        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    let tok = self.peek().clone();
                    self.error(&tok, "Can't have more than 255 parameters.");
                }

                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.match_tokens(&vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        message = String::new();
        message.push_str("Expect '{' before ");
        message.push_str(kind);
        message.push_str(" body.");
        self.consume(TokenType::LeftBrace, &message)?;

        let body = self.block_statements()?;
        Ok(Stmt::Function(name, parameters, body))
    }

    fn block(&mut self) -> Result<Stmt, ParseError> {
        let statements = self.block_statements()?;
        Ok(Stmt::Block(statements))
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_end() {
            if let Some(dec) = self.declaration() {
                statements.push(dec);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.and()?;

        while self.match_tokens(&vec![TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            exp = Expr::Logical(Box::new(exp), operator, Box::new(right));
        }

        Ok(exp)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.equality()?;

        while self.match_tokens(&vec![TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            exp = Expr::Logical(Box::new(exp), operator, Box::new(right));
        }

        Ok(exp)
    }

    // TODO: combine 'equality, comparison, term and factor'
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.comparison()?;

        // add phf for this 'tok_type's
        let tok_types = vec![TokenType::BangEqual, TokenType::EqualEqual];
        while self.match_tokens(&tok_types) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            exp = Expr::Binary(Box::new(exp), operator, Box::new(right));
        }

        Ok(exp)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.term()?;

        let tok_types = vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];
        while self.match_tokens(&tok_types) {
            let operator = self.previous().clone();
            let right = self.term()?;
            exp = Expr::Binary(Box::new(exp), operator, Box::new(right));
        }

        Ok(exp)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.factor()?;

        let tok_types = vec![TokenType::Minus, TokenType::Plus];
        while self.match_tokens(&tok_types) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            exp = Expr::Binary(Box::new(exp), operator, Box::new(right));
        }

        Ok(exp)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.unary()?;

        let tok_types = vec![TokenType::Slash, TokenType::Star];
        while self.match_tokens(&tok_types) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            exp = Expr::Binary(Box::new(exp), operator, Box::new(right));
        }

        Ok(exp)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        let tok_types = vec![TokenType::Bang, TokenType::Minus];
        if self.match_tokens(&tok_types) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.primary()?;

        loop {
            if self.match_tokens(&vec![TokenType::LeftParen]) {
                exp = self.finish_call(exp)?;
            } else {
                break;
            }
        }

        Ok(exp)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    let tok = self.peek().clone();
                    self.error(&tok, "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.match_tokens(&vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call(Box::new(callee), paren, arguments))
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        // TODO: turn to match expression
        if self.match_tokens(&vec![TokenType::False]) {
            return Ok(Expr::LiteralExpr(Literal::Bool(false)));
        }
        if self.match_tokens(&vec![TokenType::True]) {
            return Ok(Expr::LiteralExpr(Literal::Bool(true)));
        }
        if self.match_tokens(&vec![TokenType::Nil]) {
            return Ok(Expr::LiteralExpr(Literal::None));
        }

        let tok_types = vec![TokenType::Number, TokenType::String];
        if self.match_tokens(&tok_types) {
            let exp = match self.previous().get_literal().clone() {
                Literal::Number(n) => Expr::LiteralExpr(Literal::Number(n)),
                Literal::String(s) => Expr::LiteralExpr(Literal::String(s)),
                _ => {
                    let tok = self.previous().clone();
                    return Err(self.error(&tok, "Expected Number or String."));
                }
            };
            return Ok(exp);
        }

        if self.match_tokens(&vec![TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }

        if self.match_tokens(&vec![TokenType::LeftParen]) {
            let exp = self.expression()?;
            match self.consume(TokenType::RightParen, "Expect ')' after expression!") {
                Ok(_) => (),
                Err(err) => return Err(err),
            };
            return Ok(Expr::Grouping(Box::new(exp)));
        }

        let token = self.peek().clone();
        Err(self.error(&token, "Expect expression."))
        // panic!("Expected `primary` but found: {:?}.", self.previous().get_type());
    }

    fn consume(&mut self, tok_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(&tok_type) {
            return Ok(self.advance().clone());
        }

        let token = self.peek().clone();
        Err(self.error(&token, message))
    }

    fn error(&mut self, token: &Token, message: &str) -> ParseError {
        self.interpreter.error(token, message);
        ParseError
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_end() {
            if *self.previous().get_type() == TokenType::Semicolon {
                return;
            }

            let tok_types = vec![
                TokenType::Class,
                TokenType::Fun,
                TokenType::Var,
                TokenType::For,
                TokenType::If,
                TokenType::While,
                TokenType::Print,
                TokenType::Return,
            ];
            if tok_types.iter().any(|tt| tt == self.peek().get_type()) {
                return;
            }

            self.advance();
        }
    }

    fn match_tokens(&mut self, tok_types: &Vec<TokenType>) -> bool {
        for tok_type in tok_types {
            if self.check(tok_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, tok_type: &TokenType) -> bool {
        // TODO: consider using copy of TokenType
        // because it can be more efficient.
        if self.is_end() {
            return false;
        }
        *self.peek().get_type() == *tok_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_end(&self) -> bool {
        *self.peek().get_type() == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .expect("Failed peeking Token!")
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("Failed peeking previous Token!")
    }
}
