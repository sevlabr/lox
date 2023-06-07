use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;
use crate::object::Obj;
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::value::Value;
use crate::Config;
use std::{cell::RefCell, error::Error, fmt, rc::Rc};

#[derive(Debug)]
pub struct CompileError {
    message: &'static str,
}

impl Error for CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompileError: {}", self.message)
    }
}

impl CompileError {
    pub fn new(message: &'static str) -> Self {
        CompileError { message }
    }
}

enum Byte {
    Raw(u8),
    Code(OpCode),
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn inc(precedence: Self) -> Self {
        Self::try_from((precedence as u8) + 1).unwrap()
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Parser, bool) -> ()>,
    infix: Option<fn(&mut Parser) -> ()>,
    precedence: Precedence,
}

#[derive(Clone, Copy)]
struct Local {
    name: Token,
    depth: isize,
}

impl Default for Local {
    fn default() -> Self {
        Self::new(Token::default(), -10)
    }
}

impl Local {
    pub fn new(name: Token, depth: isize) -> Self {
        Self { name, depth }
    }
}

const UINT8_COUNT: usize = u8::MAX as usize + 1;

struct Compiler {
    locals: [Local; UINT8_COUNT],
    local_count: isize,
    scope_depth: isize,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new([Local::default(); UINT8_COUNT], -20, -30)
    }
}

impl Compiler {
    pub fn new(locals: [Local; UINT8_COUNT], local_count: isize, scope_depth: isize) -> Self {
        Self {
            locals,
            local_count,
            scope_depth,
        }
    }
}

pub struct Parser {
    config: Config,
    chunk: Rc<RefCell<Chunk>>,
    compiler: Compiler,
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Parser {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            chunk: Rc::new(RefCell::new(Chunk::new())),
            compiler: Compiler::default(),
            scanner: Scanner::default(),
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn set_chunk(&mut self, chunk: Rc<RefCell<Chunk>>) {
        self.chunk = chunk;
    }

    fn set_scanner(&mut self, source: String) {
        self.scanner = Scanner::new(source)
    }

    fn init_compiler(&mut self) {
        self.compiler.local_count = 0;
        self.compiler.scope_depth = 0;
    }

    fn current_chunk(&self) -> Rc<RefCell<Chunk>> {
        Rc::clone(&self.chunk)
    }

    pub fn compile(
        &mut self,
        source: String,
        chunk: Rc<RefCell<Chunk>>,
    ) -> Result<(), CompileError> {
        self.set_scanner(source);
        self.init_compiler();
        self.set_chunk(chunk);
        self.advance();

        while !self.fit(TokenType::EoF) {
            self.declaration();
        }

        self.end_compiler();

        if self.had_error {
            return Err(CompileError::new("had_error = true."));
        }
        Ok(())
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous.kind).prefix;
        let can_assign = precedence <= Precedence::Assignment;
        match prefix_rule {
            Some(func) => func(self, can_assign),
            None => {
                self.error("Expect expression.".to_string());
                return;
            }
        }

        while precedence <= self.get_rule(self.current.kind).precedence {
            self.advance();
            let infix_rule = self.get_rule(self.previous.kind).infix;
            match infix_rule {
                Some(func) => func(self),
                None => {
                    self.error("Expect valid infix handler.".to_string());
                    return;
                }
            }
        }

        if can_assign && self.fit(TokenType::Equal) {
            self.error("Invalid assignment target.".to_string());
        }
    }

    fn get_rule(&self, op: TokenType) -> ParseRule {
        match op {
            TokenType::LeftParen => ParseRule {
                prefix: Some(Parser::grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightParen => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::LeftBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Dot => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: Some(Parser::unary),
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Term,
            },
            TokenType::Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(Parser::unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: Precedence::Comparison,
            },
            TokenType::Identifier => ParseRule {
                prefix: Some(Parser::variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: Some(Parser::string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Number => ParseRule {
                prefix: Some(Parser::number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::And => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Class => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::For => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Fun => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::If => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Nil => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Or => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Print => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Return => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Super => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::This => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: Some(Parser::literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Var => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Error => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::EoF => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }

    fn emit_instruction(&self, byte: OpCode) {
        self.current_chunk()
            .borrow_mut()
            .write_instruction(byte, self.previous.line);
    }

    fn emit_raw_instruction(&self, byte: u8) {
        self.current_chunk()
            .borrow_mut()
            .write_raw_instruction(byte, self.previous.line);
    }

    fn emit_universal(&self, byte: Byte) {
        match byte {
            Byte::Raw(num) => self.emit_raw_instruction(num),
            Byte::Code(code) => self.emit_instruction(code),
        }
    }

    fn emit_instructions(&self, byte1: Byte, byte2: Byte) {
        self.emit_universal(byte1);
        self.emit_universal(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let index = Byte::Raw(self.make_constant(value));
        self.emit_instructions(Byte::Code(OpCode::Constant), index);
    }

    fn write_value(&self, value: Value) -> usize {
        self.current_chunk().borrow_mut().write_value(value)
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.write_value(value);
        if constant > u8::MAX.into() {
            self.error("Too many constants in one chunk.".to_string());
            return 0;
        }
        constant as u8
    }

    fn declaration(&mut self) {
        if self.fit(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.fit(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_instruction(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn statement(&mut self) {
        if self.fit(TokenType::Print) {
            self.print_stmt();
        } else if self.fit(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_stmt();
        }
    }

    fn print_stmt(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_instruction(OpCode::Print);
    }

    fn expression_stmt(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_instruction(OpCode::Pop);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EoF) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _: bool) {
        let op_type = self.previous.kind;

        // Compile the operand.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match op_type {
            TokenType::Bang => self.emit_instruction(OpCode::Not),
            TokenType::Minus => self.emit_instruction(OpCode::Negate),
            _ => unreachable!("Unary can be only one of: '-', '!'."),
        }
    }

    fn binary(&mut self) {
        let op_type = self.previous.kind;
        let rule = self.get_rule(op_type);
        let precedence = Precedence::inc(rule.precedence);
        self.parse_precedence(precedence);

        match op_type {
            TokenType::BangEqual => {
                self.emit_instructions(Byte::Code(OpCode::Equal), Byte::Code(OpCode::Not))
            }
            TokenType::EqualEqual => self.emit_instruction(OpCode::Equal),
            TokenType::Greater => self.emit_instruction(OpCode::Greater),
            TokenType::GreaterEqual => {
                self.emit_instructions(Byte::Code(OpCode::Less), Byte::Code(OpCode::Not))
            }
            TokenType::Less => self.emit_instruction(OpCode::Less),
            TokenType::LessEqual => {
                self.emit_instructions(Byte::Code(OpCode::Greater), Byte::Code(OpCode::Not))
            }
            TokenType::Plus => self.emit_instruction(OpCode::Add),
            TokenType::Minus => self.emit_instruction(OpCode::Subtract),
            TokenType::Star => self.emit_instruction(OpCode::Multiply),
            TokenType::Slash => self.emit_instruction(OpCode::Divide),
            _ => unreachable!(
                "Binary can be one of: '+', '-', '*', '/', '!=', '==', '>', '>=', '<', '<='."
            ),
        }
    }

    fn literal(&mut self, _: bool) {
        match self.previous.kind {
            TokenType::False => self.emit_instruction(OpCode::False),
            TokenType::Nil => self.emit_instruction(OpCode::Nil),
            TokenType::True => self.emit_instruction(OpCode::True),
            _ => unreachable!("Expected one of: 'false', 'true', 'nil'."),
        }
    }

    fn number(&mut self, _: bool) {
        let lexeme = self
            .scanner
            .lexeme(self.previous.start, self.previous.length);
        match lexeme.trim().parse::<f64>() {
            Ok(num) => self.emit_constant(Value::Num(num)),
            Err(_) => self.error("Failed parsing float number.".to_string()),
        }
    }

    fn string(&mut self, _: bool) {
        let str = self
            .scanner
            .lexeme(self.previous.start + 1, self.previous.length - 2);
        self.emit_constant(Value::Obj(Obj::Str(str)));
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg_prelim = self.resolve_local(&name);
        let (get_op, set_op, arg) = if arg_prelim != -1 {
            (
                OpCode::GetLocal,
                OpCode::SetLocal,
                arg_prelim.try_into().unwrap(),
            )
        } else {
            (
                OpCode::GetGlobal,
                OpCode::SetGlobal,
                self.identifier_constant(name),
            )
        };

        if can_assign && self.fit(TokenType::Equal) {
            self.expression();
            self.emit_instructions(Byte::Code(set_op), Byte::Raw(arg));
        } else {
            self.emit_instructions(Byte::Code(get_op), Byte::Raw(arg));
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous, can_assign);
    }

    fn identifier_constant(&mut self, token: Token) -> u8 {
        let name = self.scanner.lexeme(token.start, token.length);
        self.make_constant(Value::Obj(Obj::Str(name)))
    }

    fn parse_variable(&mut self, message: &'static str) -> u8 {
        self.consume(TokenType::Identifier, message);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }

        self.identifier_constant(self.previous)
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }

        let name = self.previous;
        let mut failed = false;
        let mut name_string = String::new();
        let mut current_depth = -100;
        for local in self
            .compiler
            .locals
            .iter()
            .take(self.compiler.local_count.try_into().unwrap())
            .rev()
        {
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }
            if self.identifiers_equal(&name, &local.name) {
                failed = true;
                name_string = self.scanner.lexeme(name.start, name.length);
                current_depth = self.compiler.scope_depth;
                break;
            }
        }
        if failed {
            self.error(format!(
                "Already a variable with this name '{}' in this scope (current depth: {}).",
                name_string, current_depth,
            ));
        }

        self.add_local(name);
    }

    fn define_variable(&mut self, var: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_instructions(Byte::Code(OpCode::DefineGlobal), Byte::Raw(var));
    }

    fn add_local(&mut self, name: Token) {
        if self.compiler.local_count >= UINT8_COUNT as isize {
            self.error(format!(
                "Too many local variables in function. Current variable count: {}.",
                self.compiler.local_count,
            ));
        }

        let local = &mut self.compiler.locals[self.compiler.local_count as usize];
        self.compiler.local_count += 1;
        local.name = name;
        local.depth = -1;
    }

    fn advance(&mut self) {
        self.previous = self.current;

        loop {
            match self.scanner.scan_token() {
                Ok(token) => {
                    self.current = token;
                    break;
                }
                Err(err) => {
                    // eprintln!("(During compilation, in advance()). {err}");
                    // let message = self.scanner.lexeme(self.current.start, self.current.length);
                    self.error_at_current(err.message());
                }
            }
        }
    }

    fn consume(&mut self, kind: TokenType, message: &'static str) {
        if self.current.kind == kind {
            self.advance();
            return;
        }

        self.error_at_current(message.to_string());
    }

    fn check(&self, kind: TokenType) -> bool {
        self.current.kind == kind
    }

    fn fit(&mut self, kind: TokenType) -> bool {
        if !self.check(kind) {
            return false;
        }
        self.advance();
        true
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        if a.length != b.length {
            return false;
        }
        let name_a = self.scanner.lexeme(a.start, a.length);
        let name_b = self.scanner.lexeme(b.start, b.length);
        name_a == name_b
    }

    fn resolve_local(&mut self, name: &Token) -> isize {
        for (i, local) in self
            .compiler
            .locals
            .iter()
            .take(self.compiler.local_count.try_into().unwrap())
            .enumerate()
            .rev()
        {
            if self.identifiers_equal(name, &local.name) {
                if local.depth == -1 {
                    self.error("Can't read local variable in its own initializer.".to_string());
                }
                return i as isize;
            }
        }

        -1
    }

    fn mark_initialized(&mut self) {
        self.compiler.locals[self.compiler.local_count as usize - 1].depth =
            self.compiler.scope_depth;
    }

    fn error_at_current(&mut self, message: String) {
        self.error_at(self.current, message);
    }

    fn error(&mut self, message: String) {
        self.error_at(self.previous, message);
    }

    fn error_at(&mut self, token: Token, message: String) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] CompileError", token.line);

        match token.kind {
            TokenType::EoF => eprint!(" at end"),
            TokenType::Error => (),
            _ => {
                let lexeme = self.scanner.lexeme(self.current.start, self.current.length);
                eprint!(" at '{}'", lexeme);
            }
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.kind != TokenType::EoF {
            if self.previous.kind == TokenType::Semicolon {
                return;
            }
            match self.current.kind {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn end_compiler(&self) {
        self.emit_return();
        if self.config.debug && self.had_error {
            disassemble_chunk(&self.chunk.borrow(), "code")
        }
    }

    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;

        while self.compiler.local_count > 0
            && self.compiler.locals[self.compiler.local_count as usize - 1].depth
                > self.compiler.scope_depth
        {
            self.emit_instruction(OpCode::Pop);
            self.compiler.local_count -= 1;
        }
    }

    fn emit_return(&self) {
        self.emit_instruction(OpCode::Return);
    }
}

/// `num_enum` crate is better solution here.
impl TryFrom<u8> for Precedence {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Precedence::None),
            1 => Ok(Precedence::Assignment),
            2 => Ok(Precedence::Or),
            3 => Ok(Precedence::And),
            4 => Ok(Precedence::Equality),
            5 => Ok(Precedence::Comparison),
            6 => Ok(Precedence::Term),
            7 => Ok(Precedence::Factor),
            8 => Ok(Precedence::Unary),
            9 => Ok(Precedence::Call),
            10 => Ok(Precedence::Primary),
            _ => {
                eprintln!("Precedence value: {}.", value);
                Err("Failed to convert from u8: unknown Precedence.")
            }
        }
    }
}
