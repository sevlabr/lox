use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;
use crate::object::{Function, Obj};
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
    is_captured: bool,
}

impl Default for Local {
    fn default() -> Self {
        Self::new(Token::default(), -10, false)
    }
}

impl Local {
    pub fn new(name: Token, depth: isize, is_captured: bool) -> Self {
        Self {
            name,
            depth,
            is_captured,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Upvalue {
    index: u8,
    is_local: bool,
}

impl Default for Upvalue {
    fn default() -> Self {
        Self::new(0, false)
    }
}

impl Upvalue {
    fn new(index: u8, is_local: bool) -> Self {
        Self { index, is_local }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum FunType {
    Function,
    Script,
}

const UINT8_COUNT: usize = u8::MAX as usize + 1;

#[derive(Clone)]
struct Compiler {
    enclosing: Option<Rc<RefCell<Self>>>,

    function: Rc<RefCell<Function>>,
    kind: FunType,

    locals: [Local; UINT8_COUNT],
    local_count: isize,
    upvalues: [Upvalue; UINT8_COUNT],
    scope_depth: isize,
}

impl Default for Compiler {
    fn default() -> Self {
        let function = Rc::new(RefCell::new(Function::new()));
        Self::new(
            None,
            function,
            FunType::Script,
            [Local::default(); UINT8_COUNT],
            -20,
            [Upvalue::default(); UINT8_COUNT],
            -30,
        )
    }
}

impl Compiler {
    pub fn new(
        enclosing: Option<Rc<RefCell<Self>>>,
        function: Rc<RefCell<Function>>,
        kind: FunType,
        locals: [Local; UINT8_COUNT],
        local_count: isize,
        upvalues: [Upvalue; UINT8_COUNT],
        scope_depth: isize,
    ) -> Self {
        Self {
            enclosing,
            function,
            kind,
            locals,
            local_count,
            upvalues,
            scope_depth,
        }
    }

    fn current_fun(&self) -> Rc<RefCell<Function>> {
        Rc::clone(&self.function)
    }

    fn set_fun_kind(&mut self, fun_kind: FunType) {
        self.kind = fun_kind;
    }

    fn set_captured(&mut self, index: isize, is_captured: bool) {
        self.locals[index as usize].is_captured = is_captured;
    }
}

pub struct Parser {
    config: Config,
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
            compiler: Compiler::default(),
            scanner: Scanner::default(),
            current: Token::default(),
            previous: Token::default(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn set_scanner(&mut self, source: String) {
        self.scanner = Scanner::new(source)
    }

    fn init_compiler(&mut self, fun_kind: FunType) {
        self.compiler.set_fun_kind(fun_kind);
        self.compiler.local_count = 0;
        self.compiler.scope_depth = 0;

        if fun_kind != FunType::Script {
            let name = self
                .scanner
                .lexeme(self.previous.start, self.previous.length);
            self.compiler.current_fun().borrow_mut().set_name(name);
        }

        let local = &mut self.compiler.locals[self.compiler.local_count as usize];
        self.compiler.local_count += 1;
        local.depth = 0;
        local.is_captured = false;
        // name == ""
        local.name.kind = TokenType::Identifier;
        local.name.start = 0;
        local.name.length = 0;
    }

    fn current_chunk(&self) -> Rc<RefCell<Chunk>> {
        Rc::clone(&self.compiler.current_fun().borrow_mut().chunk())
    }

    pub fn compile(&mut self, source: String) -> Result<Rc<RefCell<Function>>, CompileError> {
        self.set_scanner(source);
        self.init_compiler(FunType::Script);
        self.advance();

        while !self.fit(TokenType::EoF) {
            self.declaration();
        }

        let (function, _) = self.end_compiler();

        if self.had_error {
            return Err(CompileError::new("had_error = true."));
        }
        Ok(function)
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
                infix: Some(Parser::call),
                precedence: Precedence::Call,
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
                infix: Some(Parser::and_),
                precedence: Precedence::And,
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
                infix: Some(Parser::or_),
                precedence: Precedence::Or,
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

    fn emit_jump(&self, instruction: OpCode) -> isize {
        self.emit_instruction(instruction);
        self.emit_raw_instruction(255);
        self.emit_raw_instruction(255);
        self.current_chunk().borrow().code.len() as isize - 2
    }

    fn patch_jump(&mut self, offset: isize) {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = self.current_chunk().borrow().code.len() as isize - offset - 2;

        if jump > u16::MAX as isize {
            self.error("Too much code to jump over.".to_string());
        }

        self.current_chunk().borrow_mut().code[offset as usize] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().borrow_mut().code[offset as usize + 1] = (jump & 0xff) as u8;
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_instruction(OpCode::Loop);

        let offset = self.current_chunk().borrow().code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.".to_string());
        }

        self.emit_raw_instruction(((offset >> 8) & 0xff) as u8);
        self.emit_raw_instruction((offset & 0xff) as u8);
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
        if self.fit(TokenType::Fun) {
            self.fun_declaration();
        } else if self.fit(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunType::Function);
        self.define_variable(global);
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
        } else if self.fit(TokenType::For) {
            self.for_stmt();
        } else if self.fit(TokenType::If) {
            self.if_stmt();
        } else if self.fit(TokenType::Return) {
            self.return_stmt();
        } else if self.fit(TokenType::While) {
            self.while_stmt();
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

    fn if_stmt(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_instruction(OpCode::Pop);
        self.statement();

        let else_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(then_jump);
        self.emit_instruction(OpCode::Pop);

        if self.fit(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn return_stmt(&mut self) {
        if self.compiler.kind == FunType::Script {
            self.error("Can't return from top-level code.".to_string());
        }

        if self.fit(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_instruction(OpCode::Return);
        }
    }

    fn while_stmt(&mut self) {
        let loop_start = self.current_chunk().borrow().code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_instruction(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_instruction(OpCode::Pop);
    }

    fn for_stmt(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.fit(TokenType::Semicolon) {
            // No initializer.
        } else if self.fit(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_stmt();
        }

        let mut loop_start = self.current_chunk().borrow().code.len();
        let mut exit_jump: isize = -1;
        if !self.fit(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // Jump out of the loop if the condition is false.
            exit_jump = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_instruction(OpCode::Pop); // Condition.
        }

        if !self.fit(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().borrow().code.len();
            self.expression();
            self.emit_instruction(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if exit_jump != -1 {
            self.patch_jump(exit_jump);
            self.emit_instruction(OpCode::Pop); // Condition.
        }

        self.end_scope();
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EoF) {
            self.declaration();
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn function(&mut self, kind: FunType) {
        let mut compiler = Compiler::default();
        compiler.enclosing = Some(Rc::new(RefCell::new(self.compiler.clone())));
        self.compiler = compiler;
        self.init_compiler(kind);
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !self.check(TokenType::RightParen) {
            loop {
                let func = self.compiler.current_fun();
                let arity = func.borrow().arity();
                func.borrow_mut().change_arity(arity + 1);
                if arity + 1 > 255 {
                    self.error_at_current("Can't have more than 255 parameters.".to_string());
                }
                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);

                if self.fit(TokenType::Comma) {
                    continue;
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        self.block();

        let (function, compiler) = self.end_compiler();
        let index = Byte::Raw(self.make_constant(Value::Obj(Obj::Fun(function.borrow().clone()))));
        self.emit_instructions(Byte::Code(OpCode::Closure), index);

        for i in 0..function.borrow().upvalue_count() as usize {
            let is_local: u8 = match compiler.upvalues[i].is_local {
                true => 1,
                false => 0,
            };
            let index: u8 = compiler.upvalues[i].index;
            self.emit_raw_instruction(is_local);
            self.emit_raw_instruction(index);
        }
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

    fn call(&mut self) {
        let arg_count = self.argument_list();
        self.emit_instructions(Byte::Code(OpCode::Call), Byte::Raw(arg_count));
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
        let mut arg = self.resolve_local_current(&name);
        let (get_op, set_op) = if arg != -1 {
            (OpCode::GetLocal, OpCode::SetLocal)
        } else {
            let enclosing = self.compiler.enclosing.as_ref().map(Rc::clone);
            arg = self.resolve_upvalue_current(enclosing, &name);
            if arg != -1 {
                (OpCode::GetUpvalue, OpCode::SetUpvalue)
            } else {
                arg = self.identifier_constant(name) as isize;
                (OpCode::GetGlobal, OpCode::SetGlobal)
            }
        };

        let arg = arg as u8;
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

    fn argument_list(&mut self) -> u8 {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.".to_string());
                }
                arg_count += 1;

                if self.fit(TokenType::Comma) {
                    continue;
                } else {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn and_(&mut self) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);

        self.emit_instruction(OpCode::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn or_(&mut self) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);

        self.patch_jump(else_jump);
        self.emit_instruction(OpCode::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
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
        local.is_captured = false;
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

    fn resolve_local_current(&mut self, name: &Token) -> isize {
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

    fn resolve_local(&mut self, compiler: &Option<Rc<RefCell<Compiler>>>, name: &Token) -> isize {
        let current = compiler
            .as_ref()
            .expect("Enclosing compiler must not be None here.")
            .borrow();
        let local_count = current.local_count;
        for (i, local) in current
            .locals
            .iter()
            .take(local_count.try_into().unwrap())
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

    fn add_upvalue_current(&mut self, index: u8, is_local: bool) -> isize {
        let compiler = &mut self.compiler;
        let upvalue_count = compiler.function.borrow().upvalue_count() as usize;

        for (i, upvalue) in compiler.upvalues.iter().take(upvalue_count).enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i as isize;
            }
        }

        if upvalue_count == UINT8_COUNT {
            self.error("Too many closure variables in function.".to_string());
            return 0;
        }

        compiler.upvalues[upvalue_count].is_local = is_local;
        compiler.upvalues[upvalue_count].index = index;
        let upvalue_count = upvalue_count as isize;
        compiler
            .function
            .borrow_mut()
            .change_upvalue_count(upvalue_count + 1);
        upvalue_count
    }

    fn add_upvalue(&mut self, compiler: &mut Compiler, index: u8, is_local: bool) -> isize {
        let upvalue_count = compiler.function.borrow().upvalue_count() as usize;

        for (i, upvalue) in compiler.upvalues.iter().take(upvalue_count).enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i as isize;
            }
        }

        if upvalue_count == UINT8_COUNT {
            self.error("Too many closure variables in function.".to_string());
            return 0;
        }

        compiler.upvalues[upvalue_count].is_local = is_local;
        compiler.upvalues[upvalue_count].index = index;
        let upvalue_count = upvalue_count as isize;
        compiler
            .function
            .borrow_mut()
            .change_upvalue_count(upvalue_count + 1);
        upvalue_count
    }

    fn resolve_upvalue_current(
        &mut self,
        enclosing: Option<Rc<RefCell<Compiler>>>,
        name: &Token,
    ) -> isize {
        if enclosing.is_none() {
            return -1;
        }

        let local = self.resolve_local(&enclosing, name);
        if local != -1 {
            enclosing.unwrap().borrow_mut().set_captured(local, true);
            return self.add_upvalue_current(local as u8, true);
        }

        let enclosing = enclosing.unwrap();
        let enclosing = &mut *enclosing.borrow_mut();
        let enclosing_enclosing = enclosing.enclosing.as_ref().map(Rc::clone);
        let upvalue = self.resolve_upvalue(enclosing, enclosing_enclosing, name);
        if upvalue != -1 {
            return self.add_upvalue_current(upvalue as u8, false);
        }

        -1
    }

    fn resolve_upvalue(
        &mut self,
        compiler: &mut Compiler,
        enclosing: Option<Rc<RefCell<Compiler>>>,
        name: &Token,
    ) -> isize {
        if enclosing.is_none() {
            return -1;
        }

        let local = self.resolve_local(&enclosing, name);
        if local != -1 {
            enclosing.unwrap().borrow_mut().set_captured(local, true);
            return self.add_upvalue(compiler, local as u8, true);
        }

        let enclosing = enclosing.unwrap();
        let enclosing = &mut *enclosing.borrow_mut();
        let enclosing_enclosing = enclosing.enclosing.as_ref().map(Rc::clone);
        let upvalue = self.resolve_upvalue(enclosing, enclosing_enclosing, name);
        if upvalue != -1 {
            return self.add_upvalue(compiler, upvalue as u8, false);
        }

        -1
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
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

    fn end_compiler(&mut self) -> (Rc<RefCell<Function>>, Compiler) {
        self.emit_return();
        let function = self.compiler.current_fun();

        if self.config.debug && self.had_error {
            let name = if function.borrow().name().is_empty() {
                "<script>".to_string()
            } else {
                function.borrow().name()
            };
            println!();
            disassemble_chunk(&self.current_chunk().borrow(), &name)
        }

        let saved_compiler = self.compiler.clone();
        match &self.compiler.enclosing {
            Some(compiler) => {
                let compiler = compiler.borrow().clone();
                self.compiler = compiler;
            }
            None => (),
        }
        (function, saved_compiler)
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
            if self.compiler.locals[self.compiler.local_count as usize - 1].is_captured {
                self.emit_instruction(OpCode::CloseUpvalue);
            } else {
                self.emit_instruction(OpCode::Pop);
            }
            self.compiler.local_count -= 1;
        }
    }

    fn emit_return(&self) {
        self.emit_instruction(OpCode::Nil);
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
