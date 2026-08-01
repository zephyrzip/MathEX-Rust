
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetics
    Add, Sub, Mul, Div, Rem, Pow,
    // Bitwise
    Shl, Shr, BitAnd, BitOr, BitXor,
    // Logical
    Eq, Neq, Lt, Gt, Le, Ge, LogAnd, LogOr,
    // Other
    Assign, // =
    Comma,  // ,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // !
    BitNot, // ^ (MathEX uses ^ for unary bitwise negation)
}

// This is our main AST node. It represents any piece of an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    
    Number(f32),
    Variable(String),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
}