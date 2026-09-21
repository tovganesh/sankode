use sankode_core::{Expr, ExprKind, FunctionDecl, Program, Span, Statement, TopLevelItem};
use sankode_semantics::Type;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum BorrowError {
    #[error("स्वामित्व-दोषः: चरस्य '{name}' पूर्वमेव सङ्क्रमणं जातम् (Use after move) at {span} (सङ्क्रान्तम् at {moved_at})")]
    UseAfterMove {
        name: String,
        span: Span,
        moved_at: Span,
    },
    #[error("ऋण-दोषः: चरः '{name}' पूर्वमेव चलऋणेन बद्धः अस्ति (Already mutably borrowed) at {span}")]
    AlreadyMutablyBorrowed { name: String, span: Span },
    #[error("ऋण-दोषः: ऋणग्रस्तचरस्य विकारः अमान्यः (Cannot mutate '{name}' while borrowed) at {span}")]
    MutateWhileBorrowed { name: String, span: Span },
    #[error("ऋण-दोषः: सामान्यऋणे सति चलऋणं न शक्यते (Cannot borrow '{name}' as mutable while immutably borrowed) at {span}")]
    MutBorrowWhileImmutablyBorrowed { name: String, span: Span },
    #[error("ऋण-दोषः: चलऋणे सति अन्यत् ऋणं न शक्यते (Cannot borrow '{name}' while already mutably borrowed) at {span}")]
    BorrowWhileMutablyBorrowed { name: String, span: Span },
    #[error("ऋण-दोषः: सङ्क्रान्तचरस्य ऋणग्रहणं न शक्यते (Cannot borrow moved variable '{name}') at {span}")]
    BorrowMoved { name: String, span: Span },
}

#[derive(Debug, Clone)]
pub struct BindingState {
    pub ty: Type,
    pub is_mut: bool,
    pub moved_at: Option<Span>,
    pub shared_borrows: usize,
    pub is_mutably_borrowed: bool,
}

pub struct BorrowChecker {
    scopes: Vec<HashMap<String, BindingState>>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), BorrowError> {
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => self.check_function(func)?,
                TopLevelItem::Statement(stmt) => self.check_statement(stmt)?,
                TopLevelItem::Comment(_) => {}
            }
        }
        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, name: String, ty: Type, is_mut: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name,
                BindingState {
                    ty,
                    is_mut,
                    moved_at: None,
                    shared_borrows: 0,
                    is_mutably_borrowed: false,
                },
            );
        }
    }

    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut BindingState> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                return scope.get_mut(name);
            }
        }
        None
    }

    fn check_function(&mut self, func: &FunctionDecl) -> Result<(), BorrowError> {
        self.enter_scope();

        for param in &func.params {
            let ty = Type::from_annotation(&param.type_ann);
            self.define_var(param.name.clone(), ty, false);
        }

        for stmt in &func.body {
            self.check_statement(stmt)?;
        }

        self.exit_scope();
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), BorrowError> {
        match stmt {
            Statement::VarDecl {
                name,
                is_mut,
                type_ann,
                init,
                ..
            } => {
                let init_ty = if let Some(ann) = type_ann {
                    Type::from_annotation(ann)
                } else {
                    self.infer_type(init)
                };

                // Check init expression for reads and moves
                self.check_expr_read_or_move(init)?;

                self.define_var(name.clone(), init_ty, *is_mut);
                Ok(())
            }
            Statement::Assignment {
                target,
                value,
                span,
            } => {
                // Must be valid to mutate target
                if let Some(state) = self.lookup_var_mut(target) {
                    if let Some(moved_at) = state.moved_at {
                        return Err(BorrowError::UseAfterMove {
                            name: target.clone(),
                            span: *span,
                            moved_at,
                        });
                    }
                    if state.shared_borrows > 0 || state.is_mutably_borrowed {
                        return Err(BorrowError::MutateWhileBorrowed {
                            name: target.clone(),
                            span: *span,
                        });
                    }
                }

                // Check value expression
                self.check_expr_read_or_move(value)?;
                Ok(())
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.check_expr_read(condition)?;

                self.enter_scope();
                for s in then_branch {
                    self.check_statement(s)?;
                }
                self.exit_scope();

                if let Some(else_stmts) = else_branch {
                    self.enter_scope();
                    for s in else_stmts {
                        self.check_statement(s)?;
                    }
                    self.exit_scope();
                }

                Ok(())
            }
            Statement::While {
                condition, body, ..
            } => {
                self.check_expr_read(condition)?;

                self.enter_scope();
                for s in body {
                    self.check_statement(s)?;
                }
                self.exit_scope();

                Ok(())
            }
            Statement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.check_expr_read_or_move(expr)?;
                }
                Ok(())
            }
            Statement::Expr(expr) => {
                self.check_expr_read(expr)?;
                Ok(())
            }
        }
    }

    fn check_expr_read(&mut self, expr: &Expr) -> Result<(), BorrowError> {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                if let Some(state) = self.lookup_var_mut(name) {
                    if let Some(moved_at) = state.moved_at {
                        return Err(BorrowError::UseAfterMove {
                            name: name.clone(),
                            span: expr.span,
                            moved_at,
                        });
                    }
                }
                Ok(())
            }
            ExprKind::Borrow { is_mut, expr: inner } => {
                if let ExprKind::Identifier(ref name) = inner.kind {
                    if let Some(state) = self.lookup_var_mut(name) {
                        if let Some(_moved_at) = state.moved_at {
                            return Err(BorrowError::BorrowMoved {
                                name: name.clone(),
                                span: expr.span,
                            });
                        }
                        if *is_mut {
                            if state.shared_borrows > 0 {
                                return Err(BorrowError::MutBorrowWhileImmutablyBorrowed {
                                    name: name.clone(),
                                    span: expr.span,
                                });
                            }
                            if state.is_mutably_borrowed {
                                return Err(BorrowError::AlreadyMutablyBorrowed {
                                    name: name.clone(),
                                    span: expr.span,
                                });
                            }
                            state.is_mutably_borrowed = true;
                        } else {
                            if state.is_mutably_borrowed {
                                return Err(BorrowError::BorrowWhileMutablyBorrowed {
                                    name: name.clone(),
                                    span: expr.span,
                                });
                            }
                            state.shared_borrows += 1;
                        }
                    }
                } else {
                    self.check_expr_read(inner)?;
                }
                Ok(())
            }
            ExprKind::Binary { left, right, .. } => {
                self.check_expr_read(left)?;
                self.check_expr_read(right)?;
                Ok(())
            }
            ExprKind::Unary { expr: inner, .. } => self.check_expr_read(inner),
            ExprKind::Call { args, .. } => {
                for arg in args {
                    self.check_expr_read_or_move(arg)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn check_expr_read_or_move(&mut self, expr: &Expr) -> Result<(), BorrowError> {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                let is_copy = if let Some(state) = self.lookup_var_mut(name) {
                    if let Some(moved_at) = state.moved_at {
                        return Err(BorrowError::UseAfterMove {
                            name: name.clone(),
                            span: expr.span,
                            moved_at,
                        });
                    }
                    state.ty.is_copy()
                } else {
                    true
                };

                // If non-copy type, reading in assignment/argument moves ownership!
                if !is_copy {
                    if let Some(state) = self.lookup_var_mut(name) {
                        state.moved_at = Some(expr.span);
                    }
                }
                Ok(())
            }
            _ => self.check_expr_read(expr),
        }
    }

    fn infer_type(&self, expr: &Expr) -> Type {
        match &expr.kind {
            ExprKind::DevanagariInteger(_, _) => Type::Purna64,
            ExprKind::DevanagariFloat(_, _) => Type::Ansha64,
            ExprKind::StringLiteral(_) => Type::Sutra,
            ExprKind::BoolLiteral(_) => Type::Dvaidha,
            _ => Type::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sankode_lexer::Lexer;
    use sankode_parser::Parser;
    use sankode_semantics::TypeChecker;

    #[test]
    fn test_valid_borrowing_program() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान विकार्य क = १०।
    मान ख = ऋण क।
    मुद्रय("मानम् = ", क)।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();

        let mut tc = TypeChecker::new();
        tc.check_program(&program).unwrap();

        let mut bc = BorrowChecker::new();
        assert!(bc.check_program(&program).is_ok());
    }

    #[test]
    fn test_detect_use_after_move() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान सूत्र१: सूत्र = "नमस्ते"।
    मान सूत्र२ = सूत्र१।
    मुद्रय(सूत्र१)।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();

        let mut bc = BorrowChecker::new();
        let err = bc.check_program(&program).unwrap_err();
        match err {
            BorrowError::UseAfterMove { name, .. } => {
                assert_eq!(name, "सूत्र१");
            }
            _ => panic!("Expected UseAfterMove error"),
        }
    }

    #[test]
    fn test_detect_simultaneous_mutable_and_immutable_borrow() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान विकार्य क = १०।
    मान ख = ऋण क।
    मान ग = चलऋण क।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();

        let mut bc = BorrowChecker::new();
        let err = bc.check_program(&program).unwrap_err();
        match err {
            BorrowError::MutBorrowWhileImmutablyBorrowed { name, .. } => {
                assert_eq!(name, "क");
            }
            _ => panic!("Expected MutBorrowWhileImmutablyBorrowed error"),
        }
    }
}
