pub mod expr;
pub mod stmt;

use crate::lexer::token::{Literal, Token};
use crate::Visitor;
use expr::*;
use stmt::*;

pub struct AstPrinter;

#[allow(dead_code)]
enum PrintObj {
    Exp(Expr),
    St(Stmt), // Unused
    Tok(Token),
    List(Vec<PrintObj>), // Unused
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
            Expr::Binary(l, op, r) => {
                // TODO: change "&**" to smth elegant
                self.parenthesize(op.get_lexeme(), vec![&**l, &**r])
            }
            Expr::Grouping(ge) => self.parenthesize("group", vec![&**ge]),
            Expr::LiteralExpr(l) => format!("{l}"),
            Expr::Unary(op, r) => self.parenthesize(op.get_lexeme(), vec![&**r]),
            Expr::Variable(t) => t.get_lexeme().to_string(),
        }
    }

    fn visit_stmt(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::Print(exp) => self.parenthesize("print", vec![exp]),
            Stmt::Expression(exp) => self.parenthesize(";", vec![exp]),
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
    use crate::lexer::token::{Literal, Token, TokenType};

    use super::{AstPrinter, Visitor};

    #[test]
    fn test_simple_print() {
        let mut printer = AstPrinter;
        let expression = Expr::Binary(
            Box::new(Expr::Unary(
                Token::new(TokenType::Minus, "-", Literal::None, 1),
                Box::new(Expr::LiteralExpr(Literal::Number(123.0))),
            )),
            Token::new(TokenType::Star, "*", Literal::None, 1),
            Box::new(Expr::Grouping(Box::new(Expr::LiteralExpr(
                Literal::Number(45.67),
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
