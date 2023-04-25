use crate::evaluator::{Environment, Evaluator, Instance, Literal, Object, RuntimeError};
use crate::{ast::stmt::Stmt, Token, TokenType};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Stmt>,
    closure: Environment,
    is_initializer: bool,
}

impl Function {
    pub fn new(
        tok: &Token,
        declaration: Stmt,
        closure: Environment,
        is_initializer: bool,
    ) -> Result<Function, RuntimeError> {
        match declaration {
            Stmt::Function(name, parameters, body) => Ok(Function {
                name,
                parameters,
                body,
                closure,
                is_initializer,
            }),
            _ => Err(RuntimeError::new(
                tok,
                "Can create Function object only from Stmt::Function.",
            )),
        }
    }

    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    pub fn bind(&self, instance: &Instance) -> Result<Function, RuntimeError> {
        let mut environment = Environment::new(Some(Box::new(self.closure.clone())));
        environment.define("this".to_string(), Object::Instance(instance.clone()));
        let declaration = Stmt::Function(
            self.name.clone(),
            self.parameters.clone(),
            self.body.clone(),
        );
        // println!("\n\n\n\n\n\n\n\n\n{:#?}\n\n\n\n\n\n\n\n\n", environment);
        Function::new(&self.name, declaration, environment, self.is_initializer)
    }

    pub fn call(
        &mut self,
        evaluator: &mut Evaluator,
        arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        // Without recovery
        // let env = Environment::new(Some(Box::new(evaluator.environment.clone())));

        let (mut env, depth) = recover_env(evaluator.environment.clone(), self.closure.clone());

        // Debug
        // println!("{depth}");
        // println!("{:#?}", evaluator.locals);
        // crate::evaluator::environment::_print_envs(env.clone());

        for i in 0..self.parameters.len() {
            env.define(
                self.parameters.get(i).unwrap().get_lexeme().to_string(),
                arguments.get(i).unwrap().clone(),
            )
        }

        let (eval_res, closure) = evaluator.execute_block_fun(&self.body, env, depth);
        self.closure = closure;
        match eval_res {
            Ok(_) => {
                if self.is_initializer {
                    // return self.closure._get_at(0, "this");
                    return self.closure.get(Token::new(TokenType::This, "this", Literal::None, 0));
                }
                Ok(Object::None)
            },
            Err(err) => {
                if err.is_return() {
                    // The result is actually the same, if this `if` block is removed.
                    if self.is_initializer {
                        // return self.closure._get_at(0, "this");
                        return self.closure.get(Token::new(TokenType::This, "this", Literal::None, 0));
                    }
                    return Ok(err.get_value());
                }
                Err(err)
            }
        }
    }

    fn stringify(&self) -> String {
        let mut pretty_str = String::new();
        pretty_str.push_str("<fun ");
        pretty_str.push_str(self.name.get_lexeme());
        pretty_str.push('>');

        pretty_str
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stringify())
    }
}

fn recover_env(current: Environment, closure: Environment) -> (Environment, usize) {
    let mut clos = closure;
    let mut curr = current.clone();
    let mut curr_vals = curr.values();
    let mut inner_envs: Vec<Environment> = Vec::new();
    // TODO: consider case when only 'clock' (and possibly other built-ins) are in scopes
    // (they are considered the same, but they can be just temporarily empty)
    'over_closures: loop {
        for (name, _) in clos.values().iter() {
            if !curr_vals.contains_key(name) {
                match curr.enclosing() {
                    Some(boxed_env) => {
                        curr = *boxed_env.clone();
                        curr_vals = curr.values();
                    }
                    None => {
                        match clos.enclosing() {
                            Some(boxed_env) => {
                                inner_envs.push(clos);
                                clos = *boxed_env.clone();
                            }
                            None => unreachable!(
                                "Can't match global environments of closure and current environment."
                            ),
                        }
                        curr = current.clone();
                        curr_vals = curr.values();
                    }
                };
                continue 'over_closures;
            }
        }

        let depth = inner_envs.len();
        return (build_env_chain(current, inner_envs), depth);
    }
}

fn build_env_chain(origin: Environment, chain: Vec<Environment>) -> Environment {
    let mut env = Environment::new(Some(Box::new(origin)));
    for e in chain.iter().rev() {
        env.set_values(e.values());
        env = Environment::new(Some(Box::new(env)));
    }
    env
}
