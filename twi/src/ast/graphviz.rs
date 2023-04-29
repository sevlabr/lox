use super::expr::Expr;
use super::stmt::Stmt;
use crate::lexer::token::{Literal, Token};
use crate::Visitor;

fn fix_content(content: &str) -> String {
    let fixed = match content {
        "<" => "&lt;",
        ">" => "&gt;",
        "<=" => "&lt;=",
        ">=" => "&gt;=",
        _ => content,
    };
    fixed.to_string()
}

fn define_row(content: &str, mode: Option<&str>) -> String {
    let mut start = "\t\t\t<tr> <td align=\"center\">".to_string();
    if let Some(m) = mode {
        let s = format!("<{m}>");
        start.push_str(&s);
    }

    let mut end = String::new();
    if let Some(m) = mode {
        let s = format!("</{m}>");
        end.push_str(&s);
    }
    end.push_str("</td> </tr>\n");

    format!("{}{}{}", start, fix_content(content), end)
}

fn define_node(
    node_num: usize,
    color: &str,
    bold: &str,
    italic: &str,
    others: Vec<&str>,
) -> String {
    let name = format!("\tN{node_num} [\n");
    let color_def = format!("\t\tcolor=\"{color}\"\n");

    let start_table =
        "\t\tlabel=<<table border=\"0\" cellborder=\"1\" cellspacing=\"0\" cellpadding=\"4\">\n";
    let bold_row = define_row(bold, Some("b"));
    let em_row = define_row(italic, Some("i"));
    let mut other_rows = String::new();
    for other in others {
        other_rows.push_str(&define_row(other, None));
    }
    let end_table = "\t\t</table>>\n";

    let shape = "\t\tshape=plain\n";
    let end = "\t]\n";

    format!("{name}{color_def}{start_table}{bold_row}{em_row}{other_rows}{end_table}{shape}{end}")
}

fn auxiliary_node(node_num: usize, color: &str, content: &str) -> String {
    let name = format!("\tN{node_num} [\n");
    let color_def = format!("\t\tcolor=\"{color}\"\n");
    let label = format!("\t\tlabel=\"{content}\"\n");
    let end = "\t]\n";
    format!("{name}{color_def}{label}{end}")
}

fn str_type(token: &Token) -> String {
    let str_token_type = format!("{:#?}", token.get_type());
    str_token_type
}

pub struct AstVis {
    graph: String,
    current_node: usize,
}

impl Default for AstVis {
    fn default() -> Self {
        Self::new()
    }
}

impl AstVis {
    pub fn new() -> Self {
        AstVis {
            graph: String::new(),
            current_node: 0,
        }
    }

    pub fn print(&mut self, statements: Vec<Option<Stmt>>) {
        self.start_program();

        for (i, statement) in statements.iter().enumerate() {
            self.start_stmt(i);

            match statement {
                Some(stmt) => {
                    self.put_template();
                    self.process_stmt(stmt);
                }
                None => {
                    self.put_none(i);
                    eprintln!("Failed while printing AST for Graphviz.\nLine: {i}.\n(None Stmt).");
                }
            }

            self.end_graph();
        }

        self.end_graph();

        println!("{}", self.graph);
    }

    fn process_stmt(&mut self, s: &Stmt) {
        let (stmt_string, _) = self.visit_stmt(s);
        self.graph.push_str(&stmt_string)
    }

    fn put_template(&mut self) {
        let font = "\"Helvetica,Arial,sans-serif\"";
        let fontname = format!("\tfontname={font}\n");
        let node = format!("\tnode [\n\t\tstyle=filled\n\t\tshape=rect\n\t{fontname}\t]\n");
        let edge = format!("\tedge [\n\t{fontname}\t]\n");
        let s = format!("{fontname}{node}{edge}");
        self.graph.push_str(&s);
    }

    fn put_none(&mut self, count: usize) {
        let s = format!("\tNoneStmtLine{count} [color=\"#f44336\" style=filled]\n");
        self.graph.push_str(&s);
    }

    fn expr_node(&mut self, italic: &str, info: Option<Vec<&str>>) -> String {
        let others = info.unwrap_or_default();
        self.increment_node_count();
        define_node(self.current_node, "#9fc5e8", "Expr", italic, others)
    }

    fn stmt_node(&mut self, italic: &str, info: Option<Vec<&str>>) -> String {
        let others = info.unwrap_or_default();
        self.increment_node_count();
        define_node(self.current_node, "#93c47d", "Stmt", italic, others)
    }

    fn token_node(&mut self, italic: &str, info: Option<Vec<&str>>) -> String {
        let others = info.unwrap_or_default();
        self.increment_node_count();
        define_node(self.current_node, "#c27ba0", "Token", italic, others)
    }

    fn literal_node(&mut self, italic: &str, info: Option<Vec<&str>>) -> String {
        let others = info.unwrap_or_default();
        self.increment_node_count();
        define_node(self.current_node, "#ffcc60", "Literal", italic, others)
    }

    fn arguments_node(&mut self) -> String {
        self.increment_node_count();
        auxiliary_node(self.current_node, "#a88f85", "Arguments")
    }

    fn parameters_node(&mut self) -> String {
        self.increment_node_count();
        auxiliary_node(self.current_node, "#eee5de", "Parameters")
    }

    fn function_body_node(&mut self) -> String {
        self.increment_node_count();
        auxiliary_node(self.current_node, "#6ca1a2", "Function body")
    }

    fn methods_node(&mut self) -> String {
        self.increment_node_count();
        auxiliary_node(self.current_node, "#ffcde0", "Methods")
    }

    fn start_stmt(&mut self, count: usize) {
        let s = format!("subgraph stmt{count} {{\n");
        self.graph.push_str(&s);
    }

    fn start_program(&mut self) {
        self.graph.push_str("digraph Program {\n");
    }

    fn end_graph(&mut self) {
        self.graph.push_str("}\n");
    }

    fn increment_node_count(&mut self) {
        self.current_node += 1;
    }
}

impl Visitor<(String, usize), (String, usize)> for AstVis {
    fn visit_expr(&mut self, e: &Expr) -> (String, usize) {
        let next_node = self.current_node + 1;
        let expression_string = match e {
            Expr::Assign(name, value) => {
                let root = self.expr_node("Assign", Some(vec!["="]));
                let var = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                let (val, val_num) = self.visit_expr(value);
                format!(
                    "{}{}\tN{} -> N{}\n{}\tN{} -> N{}\n",
                    root,
                    var,
                    next_node,
                    next_node + 1,
                    val,
                    next_node,
                    val_num
                )
            }
            Expr::Binary(l, op, r) => {
                let root = self.expr_node("Binary", Some(vec![op.get_lexeme()]));
                let (left, nl) = self.visit_expr(l);
                let (right, nr) = self.visit_expr(r);
                format!(
                    "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                    root, left, right, next_node, nl, nr
                )
            }
            Expr::Call(callee, _, args) => {
                let root = self.expr_node("Call", None);
                let arguments = self.arguments_node();
                let (call_exp, call_num) = self.visit_expr(callee);
                let base = format!(
                    "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                    root,
                    arguments,
                    call_exp,
                    next_node,
                    next_node + 1,
                    call_num
                );

                let mut arguments = String::new();
                let mut args_nums = Vec::new();
                for arg in args {
                    let (arg_str, num) = self.visit_expr(arg);
                    arguments.push_str(&arg_str);
                    args_nums.push(num);
                }
                for i in args_nums {
                    let link = format!("\tN{} -> N{}\n", next_node + 1, i);
                    arguments.push_str(&link);
                }

                format!("{base}{arguments}")
            }
            Expr::Get(object, name) => {
                let root = self.expr_node("Get", Some(vec!["."]));
                let var = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                let (obj, obj_num) = self.visit_expr(object);
                format!(
                    "{}{}\tN{} -> N{}\n{}\tN{} -> N{}\n",
                    root,
                    var,
                    next_node,
                    next_node + 1,
                    obj,
                    next_node,
                    obj_num
                )
            }
            Expr::Grouping(group) => {
                let root = self.expr_node("Grouping", Some(vec!["()"]));
                let (inner, inner_num) = self.visit_expr(group);
                format!("{}{}\tN{} -> N{}\n", root, inner, next_node, inner_num)
            }
            Expr::LiteralExpr(literal) => match literal {
                Literal::Bool(b) => self.literal_node("Bool", Some(vec![&b.to_string()])),
                Literal::Number(n) => self.literal_node("Number", Some(vec![&n.to_string()])),
                Literal::String(s) => {
                    let mut repr = s.clone();
                    if repr.len() > 12 {
                        repr = repr[..12].to_string();
                    }
                    self.literal_node("String", Some(vec![&repr]))
                }
                Literal::None => self.literal_node("None", Some(vec!["nil"])),
            },
            Expr::Logical(l, op, r) => {
                let root = self.expr_node("Logical", Some(vec![op.get_lexeme()]));
                let (left, nl) = self.visit_expr(l);
                let (right, nr) = self.visit_expr(r);
                format!(
                    "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                    root, left, right, next_node, nl, nr
                )
            }
            Expr::Unary(op, r) => {
                let root = self.expr_node("Unary", Some(vec![op.get_lexeme()]));
                let (right, nr) = self.visit_expr(r);
                format!("{}{}\tN{} -> N{}\n", root, right, next_node, nr)
            }
            Expr::Variable(name) => {
                let root = self.expr_node("Variable", None);
                let var = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                format!("{}{}\tN{} -> N{}\n", root, var, next_node, next_node + 1)
            }
            Expr::Set(object, name, value) => {
                let root = self.expr_node("Set", Some(vec!["="]));
                let var = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                let (obj, obj_num) = self.visit_expr(object);
                let (val, val_num) = self.visit_expr(value);
                format!(
                    "{}{}\tN{} -> N{}\n{}\tN{} -> N{}\n{}\tN{} -> N{}\n",
                    root,
                    var,
                    next_node,
                    next_node + 1,
                    obj,
                    next_node,
                    obj_num,
                    val,
                    next_node,
                    val_num
                )
            }
            Expr::Super(_, method) => {
                let root = self.expr_node("Super", None);
                let attribute = self.token_node(&str_type(method), Some(vec![method.get_lexeme()]));
                format!(
                    "{}{}\tN{} -> N{}\n",
                    root,
                    attribute,
                    next_node,
                    next_node + 1
                )
            }
            Expr::This(_) => self.expr_node("This", None),
        };
        (expression_string, next_node)
    }

    fn visit_stmt(&mut self, s: &Stmt) -> (String, usize) {
        let next_node = self.current_node + 1;
        let statement_string = match s {
            Stmt::Block(stmts) => {
                let root = self.stmt_node("Block", None);

                let mut statements = String::new();
                let mut stmts_nums = Vec::new();
                for statement in stmts {
                    let (stmt_str, num) = self.visit_stmt(statement);
                    statements.push_str(&stmt_str);
                    stmts_nums.push(num);
                }
                for i in stmts_nums {
                    let link = format!("\tN{} -> N{}\n", next_node, i);
                    statements.push_str(&link);
                }

                format!("{root}{statements}")
            }
            Stmt::Class(name, superclass, methods) => {
                let root = if superclass.is_none() {
                    self.stmt_node("Class", Some(vec!["class"]))
                } else {
                    self.stmt_node("Class", Some(vec!["class", "<"]))
                };
                let class_name = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                let methods_node = self.methods_node();
                let base = if superclass.is_none() {
                    format!(
                        "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                        root,
                        class_name,
                        methods_node,
                        next_node,
                        next_node + 1,
                        next_node + 2
                    )
                } else {
                    let (supcls_node, supcls_num) = self.visit_expr(superclass.as_ref().unwrap());
                    format!(
                        "{}{}{}{}\tN{} -> {{N{}, N{}, N{}}}\n",
                        root,
                        class_name,
                        methods_node,
                        supcls_node,
                        next_node,
                        next_node + 1,
                        next_node + 2,
                        supcls_num
                    )
                };

                let mut methods_stmts = String::new();
                let mut methods_nums = Vec::new();
                for method in methods {
                    let (method_str, num) = self.visit_stmt(method);
                    methods_stmts.push_str(&method_str);
                    methods_nums.push(num);
                }
                for i in methods_nums {
                    let link = format!("\tN{} -> N{}\n", next_node + 2, i);
                    methods_stmts.push_str(&link);
                }

                format!("{base}{methods_stmts}")
            }
            Stmt::Expression(exp) => {
                let root = self.stmt_node("Expression", Some(vec![";"]));
                let (expression, expr_num) = self.visit_expr(exp);
                format!("{}{}\tN{} -> N{}\n", root, expression, next_node, expr_num)
            }
            Stmt::Function(name, parameters, body) => {
                let root = self.stmt_node("Function", Some(vec!["fun"]));
                let fun_name = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                let params_node = self.parameters_node();
                let body_node = self.function_body_node();
                let base = format!(
                    "{}{}{}{}\tN{} -> {{N{}, N{}, N{}}}\n",
                    root,
                    fun_name,
                    params_node,
                    body_node,
                    next_node,
                    next_node + 1,
                    next_node + 2,
                    next_node + 3
                );

                let mut params = String::new();
                for parameter in parameters {
                    params.push_str(
                        &self.token_node(&str_type(parameter), Some(vec![parameter.get_lexeme()])),
                    );
                }
                let n = next_node + 4;
                for i in n..n + parameters.len() {
                    let link = format!("\tN{} -> N{}\n", next_node + 2, i);
                    params.push_str(&link);
                }

                let mut func_def = String::new();
                let mut stmts_nums = Vec::new();
                for statement in body {
                    let (stmt_str, stmt_num) = self.visit_stmt(statement);
                    func_def.push_str(&stmt_str);
                    stmts_nums.push(stmt_num);
                }
                for i in stmts_nums {
                    let link = format!("\tN{} -> N{}\n", next_node + 3, i);
                    func_def.push_str(&link);
                }

                format!("{base}{params}{func_def}")
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let root = if else_branch.is_none() {
                    self.stmt_node("If", Some(vec!["if"]))
                } else {
                    self.stmt_node("If", Some(vec!["if-else"]))
                };
                let (cond, cond_num) = self.visit_expr(condition);
                let (then_stmt, then_num) = self.visit_stmt(then_branch);

                if else_branch.is_none() {
                    format!(
                        "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                        root, cond, then_stmt, next_node, cond_num, then_num
                    )
                } else {
                    let (else_stmt, else_num) = self.visit_stmt(else_branch.as_ref().unwrap());
                    format!(
                        "{}{}{}{}\tN{} -> {{N{}, N{}, N{}}}\n",
                        root, cond, then_stmt, else_stmt, next_node, cond_num, then_num, else_num
                    )
                }
            }
            Stmt::Print(exp) => {
                let root = self.stmt_node("Print", None);
                let (expression, expr_num) = self.visit_expr(exp);
                format!("{}{}\tN{} -> N{}\n", root, expression, next_node, expr_num)
            }
            Stmt::Return(_, value) => {
                let root = self.stmt_node("Return", None);
                let (val, val_num) = self.visit_expr(value);
                format!("{}{}\tN{} -> N{}\n", root, val, next_node, val_num)
            }
            Stmt::Var(name, initializer) => {
                let root = self.stmt_node("Var", Some(vec!["var", "="]));
                let var = self.token_node(&str_type(name), Some(vec![name.get_lexeme()]));
                let (val, val_num) = self.visit_expr(initializer);
                format!(
                    "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                    root,
                    var,
                    val,
                    next_node,
                    next_node + 1,
                    val_num
                )
            }
            Stmt::While(condition, body) => {
                let root = self.stmt_node("While", None);
                let (cond, cond_num) = self.visit_expr(condition);
                let (body_stmt, body_num) = self.visit_stmt(body);
                format!(
                    "{}{}{}\tN{} -> {{N{}, N{}}}\n",
                    root, cond, body_stmt, next_node, cond_num, body_num
                )
            }
        };
        (statement_string, next_node)
    }
}
