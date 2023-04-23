use crate::ast::{expr::Expr, stmt::Stmt};
use crate::lexer::token::{Literal, Token};
use crate::{Lox, Visitor};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Func,
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<Expr, usize>,
    current_function: FunctionType,

    interpreter: &'a mut Lox,
}

impl Resolver<'_> {
    pub fn new(interpreter: &mut Lox) -> Resolver {
        Resolver {
            scopes: Vec::new(),
            locals: HashMap::new(),
            interpreter,
            current_function: FunctionType::None,
        }
    }

    pub fn locals(&self) -> HashMap<Expr, usize> {
        self.locals.clone()
    }

    pub fn resolve_optional_stmts(&mut self, statements: Vec<Option<Stmt>>) {
        for statement in statements {
            self.resolve_stmt(statement.unwrap());
        }
    }

    fn resolve_stmts(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, statement: Stmt) {
        self.visit_stmt(&statement)
    }

    fn resolve_expr(&mut self, expression: Expr) {
        self.visit_expr(&expression)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();

        if scope.contains_key(name.get_lexeme()) {
            self.interpreter
                .error(&name, "Already a variable with this name in this scope.")
        }

        scope.insert(name.get_lexeme().to_string(), false);
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.get_lexeme().to_string(), true);
    }

    fn resolve_local(&mut self, exp: &Expr, name: &Token) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.get_lexeme()) {
                self.locals.insert(exp.clone(), depth);
                return;
            }
        }
    }

    fn resovle_function(&mut self, statement: &Stmt, fun_type: FunctionType) {
        if let Stmt::Function(_, params, body) = statement {
            let enclosing_fun = self.current_function;
            self.current_function = fun_type;

            self.begin_scope();
            for param in params {
                self.declare(param.clone());
                self.define(param.clone());
            }
            self.resolve_stmts(body.clone());
            self.end_scope();

            self.current_function = enclosing_fun;
        } else {
            panic!("Used function resolver for inappropriate Stmt!")
        }
    }
}

impl Visitor<(), ()> for Resolver<'_> {
    fn visit_expr(&mut self, e: &Expr) {
        match e {
            exp @ Expr::Variable(name) => {
                if !self.scopes.is_empty() {
                    if let Some(b) = self.scopes.last().unwrap().get(name.get_lexeme()) {
                        if !(*b) {
                            self.interpreter
                                .error(name, "Can't read local variable in its own initializer.");
                        }
                    }
                }

                self.resolve_local(exp, name)
            }
            exp @ Expr::Assign(name, value) => {
                self.resolve_expr(*value.clone());
                self.resolve_local(exp, name);
            }
            Expr::Binary(l, _, r) => {
                self.resolve_expr(*l.clone());
                self.resolve_expr(*r.clone());
            }
            Expr::Call(callee, _, arguments) => {
                self.resolve_expr(*callee.clone());
                for argument in arguments {
                    self.resolve_expr(argument.clone());
                }
            }
            Expr::Grouping(exp) => self.resolve_expr(*exp.clone()),
            Expr::LiteralExpr(_) => (),
            Expr::Logical(l, _, r) => {
                self.resolve_expr(*l.clone());
                self.resolve_expr(*r.clone());
            }
            Expr::Unary(_, r) => self.resolve_expr(*r.clone()),
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_stmts(statements.clone());
                self.end_scope();
            }
            Stmt::Var(name, initializer) => {
                self.declare(name.clone());
                if *initializer != Expr::LiteralExpr(Literal::None) {
                    self.resolve_expr(initializer.clone());
                }
                self.define(name.clone());
            }
            st @ Stmt::Function(name, _, _) => {
                self.declare(name.clone());
                self.define(name.clone());

                self.resovle_function(st, FunctionType::Func);
            }
            Stmt::Expression(exp) => self.resolve_expr(exp.clone()),
            Stmt::If(cond, then, els) => {
                self.resolve_expr(cond.clone());
                self.resolve_stmt(*then.clone());
                if let Some(s) = els {
                    self.resolve_stmt(*s.clone());
                }
            }
            Stmt::Print(exp) => self.resolve_expr(exp.clone()),
            Stmt::Return(keyword, value) => {
                if self.current_function == FunctionType::None {
                    self.interpreter
                        .error(keyword, "Can't return from top-level code.");
                }

                if *value != Expr::LiteralExpr(Literal::None) {
                    self.resolve_expr(value.clone());
                }
            }
            Stmt::While(cond, body) => {
                self.resolve_expr(cond.clone());
                self.resolve_stmt(*body.clone());
            }
        }
    }
}
