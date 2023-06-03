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
    prefix: Option<fn(&mut Parser) -> ()>,
    infix: Option<fn(&mut Parser) -> ()>,
    precedence: Precedence,
}

pub struct Parser {
    config: Config,
    chunk: Rc<RefCell<Chunk>>,
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

    fn current_chunk(&self) -> Rc<RefCell<Chunk>> {
        Rc::clone(&self.chunk)
    }

    pub fn compile(
        &mut self,
        source: String,
        chunk: Rc<RefCell<Chunk>>,
    ) -> Result<(), CompileError> {
        self.set_chunk(chunk);
        self.set_scanner(source);
        self.advance();
        self.expression();
        self.consume(TokenType::EoF, "Expect end of expression.");
        self.end_compiler();

        if self.had_error {
            return Err(CompileError::new("had_error = true."));
        }
        Ok(())
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = self.get_rule(self.previous.kind).prefix;
        match prefix_rule {
            Some(func) => func(self),
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
                prefix: None,
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

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
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

    fn literal(&mut self) {
        match self.previous.kind {
            TokenType::False => self.emit_instruction(OpCode::False),
            TokenType::Nil => self.emit_instruction(OpCode::Nil),
            TokenType::True => self.emit_instruction(OpCode::True),
            _ => unreachable!("Expected one of: 'false', 'true', 'nil'."),
        }
    }

    fn number(&mut self) {
        let lexeme = self
            .scanner
            .lexeme(self.previous.start, self.previous.length);
        match lexeme.trim().parse::<f64>() {
            Ok(num) => self.emit_constant(Value::Num(num)),
            Err(_) => self.error("Failed parsing float number.".to_string()),
        }
    }

    fn string(&mut self) {
        let str = self
            .scanner
            .lexeme(self.previous.start + 1, self.previous.length - 2);
        self.emit_constant(Value::Obj(Obj::Str(str)));
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

    fn end_compiler(&self) {
        self.emit_return();
        if self.config.debug && self.had_error {
            disassemble_chunk(&self.chunk.borrow(), "code")
        }
    }

    fn emit_return(&self) {
        self.emit_instruction(OpCode::Return);
    }
}

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
