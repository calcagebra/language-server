use std::collections::HashMap;

use crate::{
    ast::{Ast, Expression},
    token::Token,
};

pub struct Interpreter {
    global_vars: HashMap<String, i32>,
    scope_vars: HashMap<String, i32>,
    functions: HashMap<String, (Vec<String>, Expression)>,
    std: HashMap<String, fn(Vec<i32>) -> i32>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            global_vars: HashMap::new(),
            scope_vars: HashMap::new(),
            functions: HashMap::new(),
            std: HashMap::new(),
        }
    }

    pub fn init_std(&mut self) {
        self.std.insert("print".to_string(), |x| {
            x.iter().for_each(|a| println!("{a}"));
            0
        });
    }

    pub fn run(&mut self, ast: Vec<Ast>) {
        self.init_std();
        for node in ast {
            match node {
                Ast::Assignment(name, expr) => {
                    let r = self.parse_expression(&expr);
                    self.global_vars.insert(name.to_string(), r);
                }
                Ast::FunctionCall(i, exprs) => {
                    if self.std.get(&i).is_some() {
                        let f = self.std.get(&i).unwrap();
                        f(exprs.iter().map(|x| self.parse_expression(x)).collect());
                    } else {
                        let (args, code) = self.functions.get(&i).unwrap().clone();

                        let scope_vars = self.scope_vars.clone();

                        for (i, arg) in args.iter().enumerate() {
                            let r = self.parse_expression(&exprs[i]);
                            self.scope_vars.insert(arg.to_string(), r);
                        }

                        self.parse_expression(&code);

                        self.scope_vars = scope_vars;
                    }
                }
                Ast::FunctionDeclaration(name, args, code) => {
                    self.functions
                        .insert(name.to_string(), (args.to_vec(), code.clone()));
                }
            }
        }
    }

    pub fn parse_expression(&mut self, expr: &Expression) -> i32 {
        match expr {
            Expression::Binary(lhs, op, rhs) => match op {
                Token::Add => self.parse_expression(lhs) + self.parse_expression(rhs),
                Token::Sub => self.parse_expression(lhs) - self.parse_expression(rhs),
                Token::Mul => self.parse_expression(lhs) * self.parse_expression(rhs),
                Token::Div => self.parse_expression(lhs) / self.parse_expression(rhs),
                Token::Pow => self
                    .parse_expression(lhs)
                    .pow(self.parse_expression(rhs).try_into().unwrap()),
                _ => unimplemented!(),
            },
            Expression::Identifier(ident) => {
                if self.global_vars.get(ident).is_some() {
                    *self.global_vars.get(ident).unwrap()
                } else if self.scope_vars.get(ident).is_some() {
                    *self.scope_vars.get(ident).unwrap()
                } else {
                    panic!("attempt to access value of not assigned identifier `{ident}`")
                }
            }
            Expression::Number(n) => *n,
            Expression::FunctionCall(i, exprs) => {
                if self.std.get(i).is_some() {
                    let f = self.std.get(i).unwrap();
                    return f(exprs.iter().map(|x| self.parse_expression(x)).collect());
                }
                let (args, code) = self.functions.get(i).unwrap().clone();

                let scope_vars = self.scope_vars.clone();

                for (i, arg) in args.iter().enumerate() {
                    let r = self.parse_expression(&exprs[i]);
                    self.scope_vars.insert(arg.to_string(), r);
                }

                let r = self.parse_expression(&code);

                self.scope_vars = scope_vars;

                r
            }
        }
    }
}
