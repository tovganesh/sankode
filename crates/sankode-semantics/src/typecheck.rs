use crate::types::Type;
use sankode_core::{
    BinaryOp, Expr, ExprKind, FunctionDecl, Program, Span, Statement, TopLevelItem, UnaryOp,
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TypeError {
    #[error("प्रकारभेदः (Type mismatch): अपेक्षितः '{expected}', किन्तु प्राप्तः '{found}' at {span}")]
    Mismatch {
        expected: String,
        found: String,
        span: Span,
    },
    #[error("अज्ञातः चरः (Undefined variable) '{name}' at {span}")]
    UndefinedVar { name: String, span: Span },
    #[error("अज्ञाता क्रिया (Undefined function) '{name}' at {span}")]
    UndefinedFunc { name: String, span: Span },
    #[error("तर्कसंख्यादोषः (Argument count mismatch): अपेक्षिताः {expected}, किन्तु प्राप्ताः {found} at {span}")]
    ArgCountMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },
    #[error("अविकार्यचरस्य परिवर्तनम् अमान्यम् (Cannot mutate immutable variable) '{name}' at {span}")]
    CannotMutateImmutable { name: String, span: Span },
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: Type,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub params: Vec<Type>,
    pub return_type: Type,
    pub span: Span,
}

pub struct TypeChecker {
    functions: HashMap<String, FuncSignature>,
    scopes: Vec<HashMap<String, VarInfo>>,
    current_return_type: Type,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        // Built-in मुद्रय takes arbitrary printable arguments and returns रिक्त
        functions.insert(
            "मुद्रय".to_string(),
            FuncSignature {
                params: Vec::new(),
                return_type: Type::Rikta,
                span: Span::default(),
            },
        );

        Self {
            functions,
            scopes: vec![HashMap::new()],
            current_return_type: Type::Rikta,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        // First pass: collect function signatures
        for item in &program.items {
            if let TopLevelItem::Function(func) = item {
                let params = func
                    .params
                    .iter()
                    .map(|p| Type::from_annotation(&p.type_ann))
                    .collect();
                let return_type = func
                    .return_type
                    .as_ref()
                    .map(Type::from_annotation)
                    .unwrap_or(Type::Rikta);

                self.functions.insert(
                    func.name.clone(),
                    FuncSignature {
                        params,
                        return_type,
                        span: func.span,
                    },
                );
            }
        }

        // Second pass: check function bodies
        for item in &program.items {
            if let TopLevelItem::Function(func) = item {
                self.check_function(func)?;
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

    fn define_var(&mut self, name: String, ty: Type, is_mut: bool, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, VarInfo { ty, is_mut, span });
        }
    }

    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn check_function(&mut self, func: &FunctionDecl) -> Result<(), TypeError> {
        self.enter_scope();
        let ret_ty = func
            .return_type
            .as_ref()
            .map(Type::from_annotation)
            .unwrap_or(Type::Rikta);
        self.current_return_type = ret_ty.clone();

        for param in &func.params {
            let p_ty = Type::from_annotation(&param.type_ann);
            self.define_var(param.name.clone(), p_ty, false, param.span);
        }

        for stmt in &func.body {
            self.check_statement(stmt)?;
        }

        self.exit_scope();
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), TypeError> {
        match stmt {
            Statement::VarDecl {
                name,
                is_mut,
                type_ann,
                init,
                span,
            } => {
                let init_ty = self.check_expr(init)?;
                let final_ty = if let Some(ann) = type_ann {
                    let expected_ty = Type::from_annotation(ann);
                    if expected_ty != init_ty && init_ty != Type::Unknown {
                        return Err(TypeError::Mismatch {
                            expected: expected_ty.to_string(),
                            found: init_ty.to_string(),
                            span: init.span,
                        });
                    }
                    expected_ty
                } else {
                    init_ty
                };

                self.define_var(name.clone(), final_ty, *is_mut, *span);
                Ok(())
            }
            Statement::Assignment {
                target,
                value,
                span,
            } => {
                let var_info = self
                    .lookup_var(target)
                    .ok_or_else(|| TypeError::UndefinedVar {
                        name: target.clone(),
                        span: *span,
                    })?
                    .clone();

                if !var_info.is_mut {
                    return Err(TypeError::CannotMutateImmutable {
                        name: target.clone(),
                        span: *span,
                    });
                }

                let val_ty = self.check_expr(value)?;
                if var_info.ty != val_ty && val_ty != Type::Unknown {
                    return Err(TypeError::Mismatch {
                        expected: var_info.ty.to_string(),
                        found: val_ty.to_string(),
                        span: value.span,
                    });
                }

                Ok(())
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != Type::Dvaidha && cond_ty != Type::Purna64 && cond_ty != Type::Unknown {
                    return Err(TypeError::Mismatch {
                        expected: "द्वैध (Boolean)".to_string(),
                        found: cond_ty.to_string(),
                        span: condition.span,
                    });
                }

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
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != Type::Dvaidha && cond_ty != Type::Purna64 && cond_ty != Type::Unknown {
                    return Err(TypeError::Mismatch {
                        expected: "द्वैध (Boolean)".to_string(),
                        found: cond_ty.to_string(),
                        span: condition.span,
                    });
                }

                self.enter_scope();
                for s in body {
                    self.check_statement(s)?;
                }
                self.exit_scope();

                Ok(())
            }
            Statement::Return { value, span } => {
                let val_ty = if let Some(expr) = value {
                    self.check_expr(expr)?
                } else {
                    Type::Rikta
                };

                if self.current_return_type != val_ty && val_ty != Type::Unknown {
                    return Err(TypeError::Mismatch {
                        expected: self.current_return_type.to_string(),
                        found: val_ty.to_string(),
                        span: *span,
                    });
                }

                Ok(())
            }
            Statement::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match &expr.kind {
            ExprKind::DevanagariInteger(_, _) => Ok(Type::Purna64),
            ExprKind::DevanagariFloat(_, _) => Ok(Type::Ansha64),
            ExprKind::StringLiteral(_) => Ok(Type::Sutra),
            ExprKind::BoolLiteral(_) => Ok(Type::Dvaidha),
            ExprKind::Identifier(name) => {
                let info = self
                    .lookup_var(name)
                    .ok_or_else(|| TypeError::UndefinedVar {
                        name: name.clone(),
                        span: expr.span,
                    })?;
                Ok(info.ty.clone())
            }
            ExprKind::Unary { op, expr: inner } => {
                let inner_ty = self.check_expr(inner)?;
                match op {
                    UnaryOp::Neg => {
                        if inner_ty == Type::Purna64 || inner_ty == Type::Ansha64 {
                            Ok(inner_ty)
                        } else {
                            Err(TypeError::Mismatch {
                                expected: "संख्या (Number)".to_string(),
                                found: inner_ty.to_string(),
                                span: expr.span,
                            })
                        }
                    }
                    UnaryOp::Not => {
                        if inner_ty == Type::Dvaidha {
                            Ok(Type::Dvaidha)
                        } else {
                            Err(TypeError::Mismatch {
                                expected: "द्वैध (Boolean)".to_string(),
                                found: inner_ty.to_string(),
                                span: expr.span,
                            })
                        }
                    }
                }
            }
            ExprKind::Borrow { is_mut, expr: inner } => {
                let inner_ty = self.check_expr(inner)?;
                if *is_mut {
                    // Check if variable being mutably borrowed is declared as mutable!
                    if let ExprKind::Identifier(ref name) = inner.kind {
                        if let Some(info) = self.lookup_var(name) {
                            if !info.is_mut {
                                return Err(TypeError::CannotMutateImmutable {
                                    name: name.clone(),
                                    span: expr.span,
                                });
                            }
                        }
                    }
                    Ok(Type::MutReference(Box::new(inner_ty)))
                } else {
                    Ok(Type::Reference(Box::new(inner_ty)))
                }
            }
            ExprKind::Binary { left, op, right } => {
                let l_ty = self.check_expr(left)?;
                let r_ty = self.check_expr(right)?;

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if l_ty == r_ty && (l_ty == Type::Purna64 || l_ty == Type::Ansha64) {
                            Ok(l_ty)
                        } else if *op == BinaryOp::Add && l_ty == Type::Sutra && r_ty == Type::Sutra {
                            Ok(Type::Sutra)
                        } else {
                            Err(TypeError::Mismatch {
                                expected: format!("समाना संख्या (Compatible numbers), किन्तु '{}' एवं '{}'", l_ty, r_ty),
                                found: format!("{} {:?}", l_ty, op),
                                span: expr.span,
                            })
                        }
                    }
                    BinaryOp::Equal | BinaryOp::NotEqual => {
                        if l_ty == r_ty {
                            Ok(Type::Dvaidha)
                        } else {
                            Err(TypeError::Mismatch {
                                expected: format!("समानप्रकारौ (Matching types), किन्तु '{}' एवं '{}'", l_ty, r_ty),
                                found: format!("{} == {}", l_ty, r_ty),
                                span: expr.span,
                            })
                        }
                    }
                    BinaryOp::Less | BinaryOp::LessEq | BinaryOp::Greater | BinaryOp::GreaterEq => {
                        if l_ty == r_ty && (l_ty == Type::Purna64 || l_ty == Type::Ansha64) {
                            Ok(Type::Dvaidha)
                        } else {
                            Err(TypeError::Mismatch {
                                expected: "तुलनीया संख्या (Comparable numbers)".to_string(),
                                found: format!("{} एवं {}", l_ty, r_ty),
                                span: expr.span,
                            })
                        }
                    }
                }
            }
            ExprKind::Call { callee, args } => {
                if callee == "मुद्रय" {
                    // मुद्रय accepts any arguments
                    for arg in args {
                        self.check_expr(arg)?;
                    }
                    return Ok(Type::Rikta);
                }

                let sig = self
                    .functions
                    .get(callee)
                    .ok_or_else(|| TypeError::UndefinedFunc {
                        name: callee.clone(),
                        span: expr.span,
                    })?
                    .clone();

                if sig.params.len() != args.len() {
                    return Err(TypeError::ArgCountMismatch {
                        expected: sig.params.len(),
                        found: args.len(),
                        span: expr.span,
                    });
                }

                for (param_ty, arg_expr) in sig.params.iter().zip(args.iter()) {
                    let arg_ty = self.check_expr(arg_expr)?;
                    if param_ty != &arg_ty && arg_ty != Type::Unknown {
                        return Err(TypeError::Mismatch {
                            expected: param_ty.to_string(),
                            found: arg_ty.to_string(),
                            span: arg_expr.span,
                        });
                    }
                }

                Ok(sig.return_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sankode_lexer::Lexer;
    use sankode_parser::Parser;

    #[test]
    fn test_valid_types() {
        let code = r#"
क्रिया फिबोनाची(संख्या: पूर्ण६४) -> पूर्ण६४
    यदि संख्या <= १
        प्रति संख्या।
    इति
    प्रति फिबोनाची(संख्या - १) + फिबोनाची(संख्या - २)।
इति

क्रिया मुख्य() -> रिक्त
    मान परिणाम: पूर्ण६४ = फिबोनाची(१०)।
    मुद्रय("परिणाम = ", परिणाम)।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_type_mismatch_detected() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान क: पूर्ण६४ = "सङ्केतः"।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut checker = TypeChecker::new();
        let err = checker.check_program(&program).unwrap_err();
        match err {
            TypeError::Mismatch { expected, found, .. } => {
                assert_eq!(expected, "पूर्ण६४");
                assert_eq!(found, "सूत्र");
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_cannot_mutate_immutable() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान क = १०।
    क = २०।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut checker = TypeChecker::new();
        let err = checker.check_program(&program).unwrap_err();
        match err {
            TypeError::CannotMutateImmutable { name, .. } => {
                assert_eq!(name, "क");
            }
            _ => panic!("Expected CannotMutateImmutable error"),
        }
    }
}
