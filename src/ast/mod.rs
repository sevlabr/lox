pub mod expr;

use expr::*;

pub trait Visitor<T> {
    fn visit_expr(&self, e: &Expr) -> T;
}

struct AstPrinter;

impl Visitor<String> for AstPrinter {
    fn visit_expr(&self, e: &Expr) -> String {
        match e {
            Expr::Binary(l, op, r) => {
                // TODO: change "&**" to smth elegant
                self.parenthesize(op.lexeme(), vec![&**l, &**r])
            },
            Expr::Grouping(ge) => {
                self.parenthesize("group", vec![&**ge])
            },
            Expr::LiteralExpr(l) => format!("{l}"),
            Expr::Unary(op, r) => {
                self.parenthesize(op.lexeme(), vec![&**r])
            },
        }
    }
}

impl AstPrinter {
    fn parenthesize(&self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut pretty_expr = String::new();
        pretty_expr.push('(');
        pretty_expr.push_str(name);
        for e in exprs {
            pretty_expr.push(' ');
            pretty_expr.push_str(&self.visit_expr(&e));
        }
        pretty_expr.push(')');
        pretty_expr
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Expr;
    use crate::lexer::token::{Literal, Token, TokenType};

    use super::{AstPrinter, Visitor};

    #[test]
    fn test_simple_print() {
        let printer = AstPrinter;
        let expression = Expr::Binary(
            Box::new(Expr::Unary(
                Token::new(TokenType::Minus, "-", Literal::None, 1),
                Box::new(Expr::LiteralExpr(Literal::Number(123.0))),
            )),
            Token::new(TokenType::Star, "*", Literal::None, 1),
            Box::new(Expr::Grouping(
                Box::new(Expr::LiteralExpr(Literal::Number(45.67))),
            )),
        );
        let ref_result = "(* (- 123) (group 45.67))";
        assert_eq!(printer.visit_expr(&expression), ref_result,
                    "Failed printing simple AST!"
        );
    }
}
