use crate::util::trans_expr_to_ltl;
use smv::{Expr, Smv};

pub fn ltl_to_automata_preprocess(smv: &Smv, ltl: Expr) -> Expr {
    let ltl = smv.flatten_to_propositional_define(&ltl);
    let ltl = smv.flatten_case(ltl);
    trans_expr_to_ltl(&ltl)
}
