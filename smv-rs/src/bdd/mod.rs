use crate::{ast::Expr, Define, Smv};
use bdds::{Bdd, BddManager};
use fsmbdd::{FsmBdd, TransBddMethod};
use std::{
    collections::HashMap,
    ops::{BitAnd, BitOr, BitXor, Not},
};

#[derive(Clone)]
pub struct SmvBdd<BM: BddManager>
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    pub manager: BM,
    pub symbols: HashMap<String, usize>,
    pub defines: HashMap<String, BM::Bdd>,
    pub trans: Vec<BM::Bdd>,
    pub init: BM::Bdd,
    pub invariants: BM::Bdd,
    pub justice: Vec<BM::Bdd>,
}

pub fn expr_to_bdd<BM: BddManager>(
    manager: &BM,
    symbols: &HashMap<String, usize>,
    defines: &HashMap<String, Define>,
    defines_cache: &mut HashMap<String, BM::Bdd>,
    expr: &Expr,
) -> BM::Bdd
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    let ans = match expr {
        Expr::Ident(ident) => {
            if let Some(define) = defines.get(ident) {
                return if let Some(bdd) = defines_cache.get(ident) {
                    bdd.clone()
                } else {
                    let bdd = expr_to_bdd(manager, symbols, defines, defines_cache, &define.expr);
                    defines_cache.insert(define.ident.clone(), bdd.clone());
                    bdd
                };
            }
            manager.ith_var(symbols[ident])
        }
        Expr::LitExpr(lit) => manager.constant(*lit),
        Expr::PrefixExpr(op, sub_expr) => {
            let expr_bdd = expr_to_bdd(manager, symbols, defines, defines_cache, sub_expr);
            match op {
                crate::ast::Prefix::Not => !expr_bdd,
                crate::ast::Prefix::Next => expr_bdd.next_state(),
                _ => todo!(),
            }
        }
        Expr::InfixExpr(op, left, right) => {
            let left_bdd = expr_to_bdd(manager, symbols, defines, defines_cache, left);
            let right_bdd = expr_to_bdd(manager, symbols, defines, defines_cache, right);
            match op {
                crate::ast::Infix::And => left_bdd & right_bdd,
                crate::ast::Infix::Or => left_bdd | right_bdd,
                crate::ast::Infix::Xor => left_bdd ^ right_bdd,
                crate::ast::Infix::Imply => !left_bdd | right_bdd,
                crate::ast::Infix::Iff => !(left_bdd ^ right_bdd),
                _ => todo!(),
            }
        }
        Expr::CaseExpr(case_expr) => {
            let mut ans = expr_to_bdd(
                manager,
                symbols,
                defines,
                defines_cache,
                &case_expr.branchs.last().unwrap().1,
            );
            for i in (0..case_expr.branchs.len() - 1).rev() {
                let cond = expr_to_bdd(
                    manager,
                    symbols,
                    defines,
                    defines_cache,
                    &case_expr.branchs[i].0,
                );
                let res = expr_to_bdd(
                    manager,
                    symbols,
                    defines,
                    defines_cache,
                    &case_expr.branchs[i].1,
                );
                ans = cond.if_then_else(&res, &ans);
            }
            ans
        }
    };
    ans
}

impl<BM: BddManager> SmvBdd<BM>
where
    for<'a, 'b> &'a BM::Bdd: Not<Output = BM::Bdd>
        + BitAnd<BM::Bdd, Output = BM::Bdd>
        + BitAnd<&'b BM::Bdd, Output = BM::Bdd>
        + BitOr<BM::Bdd, Output = BM::Bdd>
        + BitOr<&'b BM::Bdd, Output = BM::Bdd>
        + BitXor<BM::Bdd, Output = BM::Bdd>
        + BitXor<&'b BM::Bdd, Output = BM::Bdd>,
{
    pub fn new(manager: &BM, smv: &Smv) -> Self {
        let mut symbols = HashMap::new();
        for i in 0..smv.vars.len() {
            let current = i * 2;
            let next = current + 1;
            assert!(symbols.insert(smv.vars[i].ident.clone(), current).is_none());
            manager.ith_var(next);
        }
        let mut defines = HashMap::new();
        let smv_define = smv.defines.clone();
        for define in smv_define {
            let bdd = expr_to_bdd(
                manager,
                &symbols,
                &smv.defines,
                &mut defines,
                &define.1.expr,
            );
            defines.insert(define.0, bdd);
        }

        let mut invariants = manager.constant(true);
        for i in 0..smv.invariants.len() {
            let expr_ddnode = expr_to_bdd(
                manager,
                &symbols,
                &smv.defines,
                &mut defines,
                &smv.invariants[i],
            );
            invariants &= expr_ddnode;
        }
        let mut trans = vec![];
        for i in 0..smv.trans.len() {
            trans.push(expr_to_bdd(
                manager,
                &symbols,
                &smv.defines,
                &mut defines,
                &smv.trans[i],
            ))
        }
        let mut init = manager.constant(true);
        for i in 0..smv.inits.len() {
            let expr_ddnode =
                expr_to_bdd(manager, &symbols, &smv.defines, &mut defines, &smv.inits[i]);
            init &= expr_ddnode;
        }
        let justice = smv
            .fairness
            .iter()
            .map(|fair| expr_to_bdd(manager, &symbols, &smv.defines, &mut defines, fair))
            .collect();
        Self {
            defines,
            manager: manager.clone(),
            symbols,
            trans,
            init,
            invariants,
            justice,
        }
    }

    pub fn to_fsmbdd(&self, method: TransBddMethod) -> FsmBdd<BM> {
        let trans = fsmbdd::Trans::new(&self.manager, self.trans.clone(), method);
        FsmBdd {
            symbols: self.symbols.clone(),
            manager: self.manager.clone(),
            init: self.init.clone(),
            invariants: self.invariants.clone(),
            trans,
            justice: self.justice.clone(),
        }
    }
}
