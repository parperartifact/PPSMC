use std::{
    fmt::Display,
    ops::{BitAnd, BitOr, Not},
};

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Prefix {
    Not,
    Next,
    LtlGlobally,
    LtlFinally,
    LtlNext,
    LtlOnce,
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Prefix::Not => "!",
            Prefix::Next => "next",
            Prefix::LtlGlobally => "[]",
            Prefix::LtlFinally => "<>",
            Prefix::LtlNext => "X",
            Prefix::LtlOnce => "O",
        };
        write!(f, "{}", display)
    }
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum Infix {
    And,
    Or,
    Xor,
    Imply,
    Iff,
    LtlUntil,
    LtlRelease,
    LtlSince,
}

impl Display for Infix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Infix::And => "&&",
            Infix::Or => "||",
            Infix::Xor => "xor",
            Infix::Imply => "->",
            Infix::Iff => "<->",
            Infix::LtlUntil => "U",
            Infix::LtlRelease => "V",
            Infix::LtlSince => "S",
        };
        write!(f, "{}", display)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct CaseExpr {
    pub branchs: Vec<(Expr, Expr)>,
}

impl Display for CaseExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut previous_all_false = Expr::LitExpr(true);
        let mut expr = Expr::LitExpr(false);
        for (cond, res) in self.branchs.iter() {
            expr = expr | (previous_all_false.clone() & cond.clone() & res.clone());
            previous_all_false = previous_all_false & !cond.clone();
        }
        write!(f, "{}", expr)
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Expr {
    Ident(String),
    LitExpr(bool),
    PrefixExpr(Prefix, Box<Expr>),
    InfixExpr(Infix, Box<Expr>, Box<Expr>),
    CaseExpr(CaseExpr),
}

impl Not for Expr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Expr::PrefixExpr(Prefix::Not, Box::new(self))
    }
}

impl BitAnd for Expr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Expr::InfixExpr(Infix::And, Box::new(self), Box::new(rhs))
    }
}

impl BitOr for Expr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Expr::InfixExpr(Infix::Or, Box::new(self), Box::new(rhs))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Ident(ident) => write!(f, "{}", ident),
            Expr::LitExpr(lit) => {
                write!(f, "{}", if *lit { "true" } else { "false" })
            }
            Expr::PrefixExpr(prefix, expr) => write!(f, "{}({})", prefix, expr),
            Expr::InfixExpr(infix, left, right) => write!(f, "({}){}({})", left, infix, right),
            Expr::CaseExpr(case_expr) => write!(f, "{}", case_expr),
        }
    }
}

impl Expr {
    pub fn partition_to_ands(self) -> Vec<Expr> {
        match self {
            Expr::InfixExpr(infix, left, right) if matches!(infix, Infix::And) => {
                let mut left = left.partition_to_ands();
                let mut right = right.partition_to_ands();
                left.append(&mut right);
                left
            }
            _ => vec![self],
        }
    }
}
