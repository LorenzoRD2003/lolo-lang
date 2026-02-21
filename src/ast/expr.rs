// TODO: Como hago para que no haya tantos Box<Expr> en los binarios. preguntar

pub enum Expr {
  Var(VarExpr),
  Const,
  Unary(UnaryExpr),
  Binary(BinaryExpr),
}

pub type VarExpr = u32; // Identificador de cada variable

enum UnaryExpr {
  Neg(Box<Expr>),
  Not(Box<Expr>),
}

enum BinaryExpr {
  Arithmetic(ArithmeticBinaryExpr),
  Comparison(ComparisonBinaryExpr),
  Logical(LogicalBinaryExpr),
}

enum ArithmeticBinaryExpr {
  Add(Box<Expr>, Box<Expr>),
  Sub(Box<Expr>, Box<Expr>),
  Mul(Box<Expr>, Box<Expr>),
  Div(Box<Expr>, Box<Expr>),
}

enum ComparisonBinaryExpr {
  Eq(Box<Expr>, Box<Expr>),
  Neq(Box<Expr>, Box<Expr>),
  Gt(Box<Expr>, Box<Expr>),
  Lt(Box<Expr>, Box<Expr>),
  Gte(Box<Expr>, Box<Expr>),
  Lte(Box<Expr>, Box<Expr>),
}

enum LogicalBinaryExpr {
  And(Box<Expr>, Box<Expr>),
  Or(Box<Expr>, Box<Expr>),
  Xor(Box<Expr>, Box<Expr>),
}
