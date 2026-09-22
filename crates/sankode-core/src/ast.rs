use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelItem {
    Function(FunctionDecl),
    Struct(StructDecl),
    Impl(ImplBlock),
    Statement(Statement),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub type_ann: TypeAnnotation,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub target: String,
    pub methods: Vec<FunctionDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ann: TypeAnnotation,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    Simple(String), // e.g., पूर्ण६४, अंश६४, सूत्र, रिक्त, बिन्दु, सूची
    Reference(Box<TypeAnnotation>), // ऋण पूर्ण६४
    MutReference(Box<TypeAnnotation>), // चलऋण पूर्ण६४
    List(Box<TypeAnnotation>), // सूची[पूर्ण६४]
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// मान [विकार्य] x [: Type] = expr।
    VarDecl {
        name: String,
        is_mut: bool,
        type_ann: Option<TypeAnnotation>,
        init: Expr,
        span: Span,
    },
    /// target = expr।
    Assignment {
        target: String,
        value: Expr,
        span: Span,
    },
    /// target.field = expr।
    FieldAssignment {
        target: String,
        field: String,
        value: Expr,
        span: Span,
    },
    /// target[index] = expr।
    IndexAssignment {
        target: Box<Expr>,
        index: Box<Expr>,
        value: Expr,
        span: Span,
    },
    /// यदि condition body [अन्यथा else_body] इति
    If {
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        span: Span,
    },
    /// यावत् condition body इति
    While {
        condition: Expr,
        body: Vec<Statement>,
        span: Span,
    },
    /// प्रति expr।
    Return {
        value: Option<Expr>,
        span: Span,
    },
    /// expr। (e.g. मुद्रय("नमस्ते")।)
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    DevanagariInteger(i64, String),
    DevanagariFloat(f64, String),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        target: Box<Expr>,
        field: String,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    Borrow {
        is_mut: bool,
        expr: Box<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Mod,      // %
    Equal,    // ==
    NotEqual, // !=
    Less,     // <
    LessEq,   // <=
    Greater,  // >
    GreaterEq,// >=
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -
    Not, // ! or न
}
