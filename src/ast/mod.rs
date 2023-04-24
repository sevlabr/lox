pub mod expr;
pub mod stmt;

use crate::lexer::token::{Literal, Token};
use crate::Visitor;
use expr::*;
use stmt::*;

pub struct AstPrinter;

enum PrintObj {
    Exp(Expr),
    St(Stmt),
    Tok(Token),
    List(Vec<PrintObj>),
}

impl Visitor<String, String> for AstPrinter {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Assign(name, value) => {
                let parts = vec![
                    PrintObj::Exp(Expr::LiteralExpr(Literal::String(
                        name.get_lexeme().to_string(),
                    ))),
                    PrintObj::Exp(*value.clone()),
                ];
                self.parenthesize_with_transform("=", &parts)
            }
            Expr::Binary(l, op, r) => self.parenthesize(op.get_lexeme(), vec![l, r]),
            Expr::Call(callee, _, arguments) => {
                let mut args: Vec<PrintObj> = Vec::new();
                for argument in arguments {
                    args.push(PrintObj::Exp(argument.clone()));
                }
                let parts = vec![PrintObj::Exp(*callee.clone()), PrintObj::List(args)];
                self.parenthesize_with_transform("call", &parts)
            }
            Expr::Grouping(ge) => self.parenthesize("group", vec![ge]),
            Expr::Get(object, name) => {
                let parts = vec![
                    PrintObj::Exp(*object.clone()),
                    PrintObj::Exp(Expr::LiteralExpr(Literal::String(
                        name.get_lexeme().to_string(),
                    ))),
                ];
                self.parenthesize_with_transform(".", &parts)
            }
            Expr::LiteralExpr(l) => format!("{l}"),
            Expr::Logical(l, op, r) => self.parenthesize(op.get_lexeme(), vec![l, r]),
            Expr::Unary(op, r) => self.parenthesize(op.get_lexeme(), vec![r]),
            Expr::Variable(t) => t.get_lexeme().to_string(),
            Expr::Set(object, name, value) => {
                let parts = vec![
                    PrintObj::Exp(*object.clone()),
                    PrintObj::Exp(Expr::LiteralExpr(Literal::String(
                        name.get_lexeme().to_string(),
                    ))),
                    PrintObj::Exp(*value.clone()),
                ];
                self.parenthesize_with_transform("=", &parts)
            }
            Expr::This(_) => "this".to_string(),
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::Print(exp) => self.parenthesize("print", vec![exp]),
            Stmt::Expression(exp) => self.parenthesize(";", vec![exp]),
            Stmt::Return(_, value) => match value {
                Expr::LiteralExpr(Literal::None) => "(return)".to_string(),
                _ => self.parenthesize("return", vec![value]),
            },
            Stmt::Var(name, initializer) => {
                if *initializer == Expr::LiteralExpr(Literal::None) {
                    let parts = vec![PrintObj::Tok(name.clone())];
                    return self.parenthesize_with_transform("var", &parts);
                }

                let parts = vec![
                    PrintObj::Tok(name.clone()),
                    PrintObj::Exp(Expr::LiteralExpr(Literal::String("=".to_string()))),
                    PrintObj::Exp(initializer.clone()),
                ];
                self.parenthesize_with_transform("var", &parts)
            }
            Stmt::Block(stmts) => {
                let mut pretty_str = String::new();
                pretty_str.push_str("block ");

                for stmt in stmts {
                    pretty_str.push_str(&self.visit_stmt(stmt));
                }

                pretty_str.push(')');
                pretty_str
            }
            Stmt::Class(name, superclass, methods) => {
                let mut pretty_str = String::new();
                pretty_str.push_str("(class ");
                pretty_str.push_str(name.get_lexeme());

                if let Some(sup) = superclass {
                    pretty_str.push_str(" < ");
                    pretty_str.push_str(&self.visit_expr(sup));
                }

                for method in methods {
                    pretty_str.push(' ');
                    pretty_str.push_str(&self.visit_stmt(method));
                }

                pretty_str.push(')');
                pretty_str
            }
            Stmt::Function(name, params, body) => {
                let mut pretty_str = String::new();
                pretty_str.push_str("(fun ");
                pretty_str.push_str(name.get_lexeme());
                pretty_str.push('(');

                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        pretty_str.push(' ');
                    }
                    pretty_str.push_str(param.get_lexeme());
                }

                pretty_str.push(')');

                for stmt in body {
                    pretty_str.push_str(&self.visit_stmt(stmt));
                }

                pretty_str.push(')');
                pretty_str
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let mut parts = vec![
                    PrintObj::Exp(condition.clone()),
                    PrintObj::St(*then_branch.clone()),
                ];
                match else_branch {
                    Some(else_stmt) => {
                        parts.push(PrintObj::St(*else_stmt.clone()));
                        self.parenthesize_with_transform("if-else", &parts)
                    }
                    None => self.parenthesize_with_transform("if", &parts),
                }
            }
            Stmt::While(condition, body) => {
                let parts = vec![
                    PrintObj::Exp(condition.clone()),
                    PrintObj::St(*body.clone()),
                ];
                self.parenthesize_with_transform("while", &parts)
            }
        }
    }
}

impl AstPrinter {
    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut pretty_expr = String::new();
        pretty_expr.push('(');
        pretty_expr.push_str(name);
        for e in exprs {
            pretty_expr.push(' ');
            pretty_expr.push_str(&self.visit_expr(e));
        }
        pretty_expr.push(')');
        pretty_expr
    }

    fn parenthesize_with_transform(&mut self, name: &str, parts: &Vec<PrintObj>) -> String {
        let mut pretty_str = String::new();
        pretty_str.push('(');
        pretty_str.push_str(name);
        self.transform(&mut pretty_str, parts);
        pretty_str.push(')');
        pretty_str
    }

    fn transform(&mut self, pretty_str: &mut String, parts: &Vec<PrintObj>) {
        for part in parts {
            pretty_str.push(' ');
            match part {
                PrintObj::Exp(exp) => {
                    pretty_str.push_str(&self.visit_expr(exp));
                }
                PrintObj::St(s) => {
                    pretty_str.push_str(&self.visit_stmt(s));
                }
                PrintObj::Tok(t) => {
                    pretty_str.push_str(t.get_lexeme());
                }
                PrintObj::List(lst) => self.transform(pretty_str, lst),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Expr;
    use crate::lexer::token::{Literal, Num, Token, TokenType};

    use super::{AstPrinter, Visitor};

    #[test]
    fn test_simple_print() {
        let mut printer = AstPrinter;
        let expression = Expr::Binary(
            Box::new(Expr::Unary(
                Token::new(TokenType::Minus, "-", Literal::None, 1),
                Box::new(Expr::LiteralExpr(Literal::Number(Num::new(123.0)))),
            )),
            Token::new(TokenType::Star, "*", Literal::None, 1),
            Box::new(Expr::Grouping(Box::new(Expr::LiteralExpr(
                Literal::Number(Num::new(45.67)),
            )))),
        );
        let ref_result = "(* (- 123) (group 45.67))";
        assert_eq!(
            printer.visit_expr(&expression),
            ref_result,
            "Failed printing simple AST!"
        );
    }
}
