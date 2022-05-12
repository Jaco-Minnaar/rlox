pub mod expr;

use self::expr::Expr;
use std::fmt::{self, Display, Formatter};

macro_rules! parenthesize {
    ( $s: tt, $($x:expr),* ) => {
        {
            let mut builder = String::from('(');
            builder.push_str(format!("{}", $s).as_str());
            $(
                builder.push_str(format!(" {}", $x).as_str());
            )*
            builder.push_str(")");

            builder
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::expr::ExprKind::*;
        let result = match &self.kind {
            Binary(op, lhs, rhs) => parenthesize!(op, lhs, rhs),
            Grouping(expr) => parenthesize!("group", expr),
            Literal(lit) => format!("{}", lit),
            Unary(op, expr) => parenthesize!(op, expr),
        };

        write!(f, "{}", result)
    }
}

pub fn pretty_print(expr: &Expr) -> String {
    format!("{}", expr)
}
