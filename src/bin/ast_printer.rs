use rlox::ast::{
    self,
    expr::{BinOp, Expr, ExprKind, Literal, UnOp},
};

fn main() {
    let expr = Expr {
        kind: ExprKind::Binary(
            BinOp::Multiply,
            Box::new(Expr {
                kind: ExprKind::Unary(
                    UnOp::BinNeg,
                    Box::new(Expr {
                        kind: ExprKind::Literal(Literal::Number(123.0)),
                    }),
                ),
            }),
            Box::new(Expr {
                kind: ExprKind::Grouping(Box::new(Expr {
                    kind: ExprKind::Literal(Literal::Number(45.67)),
                })),
            }),
        ),
    };

    let output = ast::pretty_print(&expr);
    println!("{}", output);
}
