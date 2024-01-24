mod ast;
pub mod bdd;
mod lexer;
mod parser;
mod token;

pub use ast::*;

use crate::{parser::parse_tokens, token::Tokens};
use lexer::lex_tokens;
use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    io,
    mem::take,
    ops::{Add, AddAssign},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct Define {
    pub ident: String,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub ident: String,
}

#[derive(Default, Debug, Clone)]
pub struct Smv {
    pub defines: HashMap<String, Define>,
    pub vars: Vec<Var>,
    pub inits: Vec<Expr>,
    pub trans: Vec<Expr>,
    pub invariants: Vec<Expr>,
    pub fairness: Vec<Expr>,
    pub ltlspecs: Vec<Expr>,
}

impl Smv {
    fn flatten_expr(&mut self, expr: Expr, flattened: &mut HashSet<String>) -> Expr {
        match expr {
            Expr::LitExpr(_) => expr,
            Expr::Ident(ident) => {
                if let Some(define) = self.defines.get(&ident) {
                    if !flattened.contains(&ident) {
                        let mut define = define.clone();
                        define.expr = self.flatten_expr(define.expr, flattened);
                        self.defines.insert(ident.clone(), define);
                        flattened.insert(ident.clone());
                        return self.defines.get(&ident).unwrap().expr.clone();
                    } else {
                        return define.expr.clone();
                    }
                }
                for latch in self.vars.iter() {
                    if latch.ident == ident {
                        return Expr::Ident(ident);
                    }
                }
                panic!()
            }
            Expr::PrefixExpr(op, sub_expr) => {
                Expr::PrefixExpr(op, Box::new(self.flatten_expr(*sub_expr, flattened)))
            }
            Expr::InfixExpr(op, left, right) => Expr::InfixExpr(
                op,
                Box::new(self.flatten_expr(*left, flattened)),
                Box::new(self.flatten_expr(*right, flattened)),
            ),
            Expr::CaseExpr(case_expr) => {
                // case_expr.branchs = case_expr
                //     .branchs
                //     .into_iter()
                //     .map(|(x, y)| (self.flatten_expr(x), self.flatten_expr(y)))
                //     .collect();
                // Expr::CaseExpr(case_expr)
                let mut ans =
                    self.flatten_expr(case_expr.branchs.last().unwrap().1.clone(), flattened);
                for i in (0..case_expr.branchs.len() - 1).rev() {
                    let cond = self.flatten_expr(case_expr.branchs[i].0.clone(), flattened);
                    let res = self.flatten_expr(case_expr.branchs[i].1.clone(), flattened);
                    ans = (cond.clone() & res) | (!cond & ans);
                }
                ans
            }
        }
    }

    fn dedup(&mut self) {
        let trans = take(&mut self.trans);
        for tran in trans {
            if !self.trans.contains(&tran) {
                self.trans.push(tran);
            }
        }
    }
}

impl Smv {
    fn parse(input: &str) -> Self {
        let tokens = lex_tokens(input).unwrap();
        let tokens = Tokens::new(&tokens);
        let mut smv = parse_tokens(tokens).unwrap();
        smv.dedup();
        smv
    }

    pub fn from_file<P: AsRef<Path>>(file: P) -> io::Result<Self> {
        let s = read_to_string(file)?;
        Ok(Self::parse(&s))
    }

    pub fn flatten_defines(&self) -> Self {
        let mut res = self.clone();
        let mut flattend = HashSet::new();
        for i in 0..res.inits.len() {
            res.inits[i] = res.flatten_expr(res.inits[i].clone(), &mut flattend);
        }
        for i in 0..res.trans.len() {
            res.trans[i] = res.flatten_expr(res.trans[i].clone(), &mut flattend);
        }
        for i in 0..res.invariants.len() {
            res.invariants[i] = res.flatten_expr(res.invariants[i].clone(), &mut flattend);
        }
        for i in 0..res.fairness.len() {
            res.fairness[i] = res.flatten_expr(res.fairness[i].clone(), &mut flattend);
        }
        for i in 0..res.ltlspecs.len() {
            res.ltlspecs[i] = res.flatten_expr(res.ltlspecs[i].clone(), &mut flattend);
        }
        res
    }

    fn flatten_to_propositional_define_rec(&self, expr: &Expr) -> Option<Expr> {
        match expr {
            Expr::Ident(ident) => {
                if let Some(define) = self.defines.get(ident) {
                    self.flatten_to_propositional_define_rec(&define.expr)
                } else {
                    None
                }
            }
            Expr::LitExpr(_) => None,
            Expr::PrefixExpr(prefix, sub_expr) => {
                let sub_expr = if let Some(sub) = self.flatten_to_propositional_define_rec(sub_expr)
                {
                    sub
                } else {
                    if let Prefix::Not = *prefix {
                        return None;
                    }
                    *sub_expr.clone()
                };
                Some(Expr::PrefixExpr(prefix.clone(), Box::new(sub_expr)))
            }
            Expr::InfixExpr(infix, left, right) => {
                let left_flatten = self.flatten_to_propositional_define_rec(left);
                let right_flatten = self.flatten_to_propositional_define_rec(right);
                let (left, right) = match (left_flatten, right_flatten) {
                    (None, None) => {
                        if let Infix::And | Infix::Or | Infix::Iff | Infix::Imply = *infix {
                            return None;
                        }
                        (*left.clone(), *right.clone())
                    }
                    (None, Some(r)) => (*left.clone(), r),
                    (Some(l), None) => (l, *right.clone()),
                    (Some(l), Some(r)) => (l, r),
                };
                Some(Expr::InfixExpr(
                    infix.clone(),
                    Box::new(left),
                    Box::new(right),
                ))
            }
            Expr::CaseExpr(case_expr) => {
                let mut update = false;
                let mut res = Vec::new();
                for (condition, branch) in case_expr.branchs.iter() {
                    let condition = if let Some(condition) =
                        self.flatten_to_propositional_define_rec(condition)
                    {
                        update = true;
                        condition
                    } else {
                        condition.clone()
                    };
                    let branch =
                        if let Some(branch) = self.flatten_to_propositional_define_rec(branch) {
                            update = true;
                            branch
                        } else {
                            branch.clone()
                        };
                    res.push((condition, branch))
                }
                if !update {
                    return None;
                }
                Some(Expr::CaseExpr(CaseExpr { branchs: res }))
            }
        }
    }

    pub fn flatten_to_propositional_define(&self, expr: &Expr) -> Expr {
        if let Some(res) = self.flatten_to_propositional_define_rec(expr) {
            res
        } else {
            expr.clone()
        }
    }

    pub fn flatten_case(&self, expr: Expr) -> Expr {
        match expr {
            Expr::Ident(_) | Expr::LitExpr(_) => expr,
            Expr::PrefixExpr(op, sub_expr) => {
                Expr::PrefixExpr(op, Box::new(self.flatten_case(*sub_expr)))
            }
            Expr::InfixExpr(op, left, right) => Expr::InfixExpr(
                op,
                Box::new(self.flatten_case(*left)),
                Box::new(self.flatten_case(*right)),
            ),
            Expr::CaseExpr(case_expr) => {
                let mut ans = self.flatten_case(case_expr.branchs.last().unwrap().1.clone());
                for i in (0..case_expr.branchs.len() - 1).rev() {
                    let cond = self.flatten_case(case_expr.branchs[i].0.clone());
                    let res = self.flatten_case(case_expr.branchs[i].1.clone());
                    ans = (cond.clone() & res) | (!cond & ans);
                }
                ans
            }
        }
    }
}

impl Add for Smv {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Smv {
    fn add_assign(&mut self, rhs: Self) {
        self.defines.extend(rhs.defines);
        self.vars.extend(rhs.vars);
        self.inits.extend(rhs.inits);
        self.trans.extend(rhs.trans);
        self.invariants.extend(rhs.invariants);
        self.fairness.extend(rhs.fairness);
        self.ltlspecs.extend(rhs.ltlspecs);
    }
}
