use smv::{Expr, Prefix};

fn trans_expr_to_ltl_rec(expr: &Expr) -> Expr {
    match expr {
        Expr::PrefixExpr(prefix, expr) => match prefix {
            Prefix::Next => {
                Expr::PrefixExpr(Prefix::LtlNext, Box::new(trans_expr_to_ltl_rec(expr)))
            }
            _ => Expr::PrefixExpr(prefix.clone(), Box::new(trans_expr_to_ltl_rec(expr))),
        },
        Expr::Ident(_) => expr.clone(),
        Expr::LitExpr(_) => expr.clone(),
        Expr::CaseExpr(_) => todo!(),
        Expr::InfixExpr(infix, left, right) => Expr::InfixExpr(
            infix.clone(),
            Box::new(trans_expr_to_ltl_rec(left)),
            Box::new(trans_expr_to_ltl_rec(right)),
        ),
    }
}

pub fn trans_expr_to_ltl(expr: &Expr) -> Expr {
    trans_expr_to_ltl_rec(expr)
}
