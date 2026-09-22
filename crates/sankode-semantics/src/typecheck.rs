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
    #[error("अज्ञाता संरचना (Undefined struct) '{name}' at {span}")]
    UndefinedStruct { name: String, span: Span },
    #[error("अज्ञातं क्षेत्रम् (Undefined field) '{field}' in struct '{struct_name}' at {span}")]
    UndefinedField { struct_name: String, field: String, span: Span },
    #[error("अज्ञाता विधिः (Undefined method) '{method}' for struct '{struct_name}' at {span}")]
    UndefinedMethod { struct_name: String, method: String, span: Span },
    #[error("अनुपस्थितं क्षेत्रम् (Missing field) '{field}' in struct '{struct_name}' at {span}")]
    MissingStructField { struct_name: String, field: String, span: Span },
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: Type,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: HashMap<String, Type>,
    pub field_order: Vec<String>,
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
    structs: HashMap<String, StructInfo>,
    methods: HashMap<(String, String), FuncSignature>,
    scopes: Vec<HashMap<String, VarInfo>>,
    current_return_type: Type,
}

fn types_compatible(expected: &Type, actual: &Type) -> bool {
    if expected == actual || expected == &Type::Unknown || actual == &Type::Unknown {
        return true;
    }
    match (expected, actual) {
        (Type::SoochiAny, Type::Soochi(_)) | (Type::Soochi(_), Type::SoochiAny) => true,
        (Type::Reference(a), b) | (b, Type::Reference(a)) if a.as_ref() == b => true,
        _ => false,
    }
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
        functions.insert(
            "सूची_सृज".to_string(),
            FuncSignature {
                params: vec![Type::Purna64, Type::Unknown],
                return_type: Type::SoochiAny,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूची_दैर्घ्यम्".to_string(),
            FuncSignature {
                params: vec![Type::SoochiAny],
                return_type: Type::Purna64,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूची_संयोजय".to_string(),
            FuncSignature {
                params: vec![Type::SoochiAny, Type::Unknown],
                return_type: Type::Rikta,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूत्र_दैर्घ्यम्".to_string(),
            FuncSignature {
                params: vec![Type::Sutra],
                return_type: Type::Purna64,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूत्र_वर्ण".to_string(),
            FuncSignature {
                params: vec![Type::Sutra, Type::Purna64],
                return_type: Type::Sutra,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूत्र_अंश".to_string(),
            FuncSignature {
                params: vec![Type::Sutra, Type::Purna64, Type::Purna64],
                return_type: Type::Sutra,
                span: Span::default(),
            },
        );
        functions.insert(
            "लॉग".to_string(),
            FuncSignature {
                params: vec![Type::Ansha64],
                return_type: Type::Ansha64,
                span: Span::default(),
            },
        );
        functions.insert(
            "घाताङ्क".to_string(),
            FuncSignature {
                params: vec![Type::Ansha64],
                return_type: Type::Ansha64,
                span: Span::default(),
            },
        );
        functions.insert(
            "वर्गमूल".to_string(),
            FuncSignature {
                params: vec![Type::Ansha64],
                return_type: Type::Ansha64,
                span: Span::default(),
            },
        );
        functions.insert(
            "पूर्णाङ्क".to_string(),
            FuncSignature {
                params: vec![Type::Ansha64],
                return_type: Type::Purna64,
                span: Span::default(),
            },
        );
        functions.insert(
            "अंशाङ्क".to_string(),
            FuncSignature {
                params: vec![Type::Purna64],
                return_type: Type::Ansha64,
                span: Span::default(),
            },
        );
        functions.insert(
            "संचिका_पठ".to_string(),
            FuncSignature {
                params: vec![Type::Sutra],
                return_type: Type::Sutra,
                span: Span::default(),
            },
        );
        functions.insert(
            "संचिका_लेख".to_string(),
            FuncSignature {
                params: vec![Type::Sutra, Type::Sutra],
                return_type: Type::Rikta,
                span: Span::default(),
            },
        );
        functions.insert(
            "संचिका_लिख".to_string(),
            FuncSignature {
                params: vec![Type::Sutra, Type::Sutra],
                return_type: Type::Rikta,
                span: Span::default(),
            },
        );
        functions.insert(
            "संचिका_विद्यते".to_string(),
            FuncSignature {
                params: vec![Type::Sutra],
                return_type: Type::Dvaidha,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूत्र_विभाजय".to_string(),
            FuncSignature {
                params: vec![Type::Sutra, Type::Sutra],
                return_type: Type::Soochi(Box::new(Type::Sutra)),
                span: Span::default(),
            },
        );
        functions.insert(
            "सूची_संयोग".to_string(),
            FuncSignature {
                params: vec![Type::SoochiAny, Type::Sutra],
                return_type: Type::Sutra,
                span: Span::default(),
            },
        );
        functions.insert(
            "संख्या_पाठ".to_string(),
            FuncSignature {
                params: vec![Type::Sutra],
                return_type: Type::Ansha64,
                span: Span::default(),
            },
        );
        functions.insert(
            "सूत्र_रूप".to_string(),
            FuncSignature {
                params: vec![Type::Unknown],
                return_type: Type::Sutra,
                span: Span::default(),
            },
        );

        Self {
            functions,
            structs: HashMap::new(),
            methods: HashMap::new(),
            scopes: vec![HashMap::new()],
            current_return_type: Type::Rikta,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        // Pass 1: collect structs
        for item in &program.items {
            if let TopLevelItem::Struct(s) = item {
                let mut fields = HashMap::new();
                let mut field_order = Vec::new();
                for f in &s.fields {
                    let f_ty = Type::from_annotation(&f.type_ann);
                    fields.insert(f.name.clone(), f_ty);
                    field_order.push(f.name.clone());
                }
                self.structs.insert(
                    s.name.clone(),
                    StructInfo {
                        name: s.name.clone(),
                        fields,
                        field_order,
                        span: s.span,
                    },
                );
            }
        }

        // Pass 2: collect function and method signatures
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => {
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
                TopLevelItem::Impl(imp) => {
                    for method in &imp.methods {
                        let mut params = Vec::new();
                        for p in &method.params {
                            let p_ty = if p.name == "स्व" {
                                match &p.type_ann {
                                    sankode_core::TypeAnnotation::Reference(_) => {
                                        Type::Reference(Box::new(Type::Struct(imp.target.clone())))
                                    }
                                    sankode_core::TypeAnnotation::MutReference(_) => {
                                        Type::MutReference(Box::new(Type::Struct(imp.target.clone())))
                                    }
                                    _ => Type::Struct(imp.target.clone()),
                                }
                            } else {
                                Type::from_annotation(&p.type_ann)
                            };
                            params.push(p_ty);
                        }
                        let return_type = method
                            .return_type
                            .as_ref()
                            .map(Type::from_annotation)
                            .unwrap_or(Type::Rikta);

                        self.methods.insert(
                            (imp.target.clone(), method.name.clone()),
                            FuncSignature {
                                params,
                                return_type,
                                span: method.span,
                            },
                        );
                    }
                }
                _ => {}
            }
        }

        // Pass 3: check function bodies, impl methods, and top-level statements
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => self.check_function(func)?,
                TopLevelItem::Impl(imp) => {
                    for method in &imp.methods {
                        self.check_method(&imp.target, method)?;
                    }
                }
                TopLevelItem::Struct(_) => {}
                TopLevelItem::Statement(stmt) => self.check_statement(stmt)?,
                TopLevelItem::Comment(_) => {}
            }
        }

        Ok(())
    }

    fn check_method(&mut self, target: &str, method: &FunctionDecl) -> Result<(), TypeError> {
        let sig = self
            .methods
            .get(&(target.to_string(), method.name.clone()))
            .unwrap()
            .clone();

        self.enter_scope();
        let prev_ret = self.current_return_type.clone();
        self.current_return_type = sig.return_type.clone();

        for (param, ty) in method.params.iter().zip(sig.params.iter()) {
            let is_mut = matches!(ty, Type::MutReference(_));
            self.define_var(param.name.clone(), ty.clone(), is_mut, param.span);
        }

        for stmt in &method.body {
            self.check_statement(stmt)?;
        }

        self.current_return_type = prev_ret;
        self.exit_scope();

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
                    if !types_compatible(&expected_ty, &init_ty) {
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
                if !types_compatible(&var_info.ty, &val_ty) {
                    return Err(TypeError::Mismatch {
                        expected: var_info.ty.to_string(),
                        found: val_ty.to_string(),
                        span: value.span,
                    });
                }

                Ok(())
            }
            Statement::FieldAssignment {
                target,
                field,
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

                let struct_name = match &var_info.ty {
                    Type::Struct(name) => name.clone(),
                    Type::MutReference(inner) => match &**inner {
                        Type::Struct(name) => name.clone(),
                        _ => {
                            return Err(TypeError::Mismatch {
                                expected: "संरचना (Struct)".to_string(),
                                found: var_info.ty.to_string(),
                                span: *span,
                            });
                        }
                    },
                    _ => {
                        return Err(TypeError::Mismatch {
                            expected: "संरचना (Struct)".to_string(),
                            found: var_info.ty.to_string(),
                            span: *span,
                        });
                    }
                };

                let s_info = self
                    .structs
                    .get(&struct_name)
                    .ok_or_else(|| TypeError::UndefinedStruct {
                        name: struct_name.clone(),
                        span: *span,
                    })?
                    .clone();

                let expected_field_ty = s_info
                    .fields
                    .get(field)
                    .ok_or_else(|| TypeError::UndefinedField {
                        struct_name: struct_name.clone(),
                        field: field.clone(),
                        span: *span,
                    })?;

                let val_ty = self.check_expr(value)?;
                if !types_compatible(expected_field_ty, &val_ty) {
                    return Err(TypeError::Mismatch {
                        expected: expected_field_ty.to_string(),
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
            Statement::IndexAssignment {
                target,
                index,
                value,
                span,
            } => {
                let target_ty = self.check_expr(target)?;
                let idx_ty = self.check_expr(index)?;
                if idx_ty != Type::Purna64 && idx_ty != Type::Unknown {
                    return Err(TypeError::Mismatch {
                        expected: "पूर्ण६४ (Integer index)".to_string(),
                        found: idx_ty.to_string(),
                        span: index.span,
                    });
                }
                let val_ty = self.check_expr(value)?;
                match target_ty {
                    Type::Soochi(ref elem_ty) => {
                        if !types_compatible(elem_ty, &val_ty) {
                            return Err(TypeError::Mismatch {
                                expected: elem_ty.to_string(),
                                found: val_ty.to_string(),
                                span: value.span,
                            });
                        }
                        Ok(())
                    }
                    Type::SoochiAny | Type::Unknown => Ok(()),
                    other => Err(TypeError::Mismatch {
                        expected: "सूची (List)".to_string(),
                        found: other.to_string(),
                        span: *span,
                    }),
                }
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
            ExprKind::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    Ok(Type::SoochiAny)
                } else {
                    let first_ty = self.check_expr(&elements[0])?;
                    for el in &elements[1..] {
                        let el_ty = self.check_expr(el)?;
                        if !types_compatible(&first_ty, &el_ty) {
                            return Err(TypeError::Mismatch {
                                expected: first_ty.to_string(),
                                found: el_ty.to_string(),
                                span: el.span,
                            });
                        }
                    }
                    Ok(Type::Soochi(Box::new(first_ty)))
                }
            }
            ExprKind::Index { target, index } => {
                let target_ty = self.check_expr(target)?;
                let idx_ty = self.check_expr(index)?;
                if idx_ty != Type::Purna64 && idx_ty != Type::Unknown {
                    return Err(TypeError::Mismatch {
                        expected: "पूर्ण६४ (Integer index)".to_string(),
                        found: idx_ty.to_string(),
                        span: index.span,
                    });
                }
                match target_ty {
                    Type::Soochi(elem_ty) => Ok(*elem_ty),
                    Type::SoochiAny => Ok(Type::Unknown),
                    Type::Sutra => Ok(Type::Sutra),
                    other => Err(TypeError::Mismatch {
                        expected: "सूची वा सूत्र (List or String)".to_string(),
                        found: other.to_string(),
                        span: target.span,
                    }),
                }
            }
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
                        if (l_ty == r_ty && (l_ty == Type::Purna64 || l_ty == Type::Ansha64))
                            || l_ty == Type::Unknown
                            || r_ty == Type::Unknown
                        {
                            let res = if l_ty != Type::Unknown { l_ty } else { r_ty };
                            Ok(res)
                        } else if *op == BinaryOp::Add && (l_ty == Type::Sutra || r_ty == Type::Sutra) {
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
                        if types_compatible(&l_ty, &r_ty) {
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
                        if (l_ty == r_ty && (l_ty == Type::Purna64 || l_ty == Type::Ansha64))
                            || l_ty == Type::Unknown
                            || r_ty == Type::Unknown
                        {
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
                    if !types_compatible(param_ty, &arg_ty) {
                        return Err(TypeError::Mismatch {
                            expected: param_ty.to_string(),
                            found: arg_ty.to_string(),
                            span: arg_expr.span,
                        });
                    }
                }

                Ok(sig.return_type)
            }
            ExprKind::FieldAccess { target, field } => {
                let target_ty = self.check_expr(target)?;
                let struct_name = match &target_ty {
                    Type::Struct(name) => name.clone(),
                    Type::Reference(inner) | Type::MutReference(inner) => match &**inner {
                        Type::Struct(name) => name.clone(),
                        _ => {
                            return Err(TypeError::Mismatch {
                                expected: "संरचना (Struct)".to_string(),
                                found: target_ty.to_string(),
                                span: expr.span,
                            });
                        }
                    },
                    _ => {
                        return Err(TypeError::Mismatch {
                            expected: "संरचना (Struct)".to_string(),
                            found: target_ty.to_string(),
                            span: expr.span,
                        });
                    }
                };

                let s_info = self
                    .structs
                    .get(&struct_name)
                    .ok_or_else(|| TypeError::UndefinedStruct {
                        name: struct_name.clone(),
                        span: expr.span,
                    })?;

                let field_ty = s_info
                    .fields
                    .get(field)
                    .ok_or_else(|| TypeError::UndefinedField {
                        struct_name: struct_name.clone(),
                        field: field.clone(),
                        span: expr.span,
                    })?;

                Ok(field_ty.clone())
            }
            ExprKind::MethodCall {
                target,
                method,
                args,
            } => {
                let target_ty = self.check_expr(target)?;
                let struct_name = match &target_ty {
                    Type::Struct(name) => name.clone(),
                    Type::Reference(inner) | Type::MutReference(inner) => match &**inner {
                        Type::Struct(name) => name.clone(),
                        _ => {
                            return Err(TypeError::Mismatch {
                                expected: "संरचना (Struct)".to_string(),
                                found: target_ty.to_string(),
                                span: expr.span,
                            });
                        }
                    },
                    _ => {
                        return Err(TypeError::Mismatch {
                            expected: "संरचना (Struct)".to_string(),
                            found: target_ty.to_string(),
                            span: expr.span,
                        });
                    }
                };

                let sig = self
                    .methods
                    .get(&(struct_name.clone(), method.clone()))
                    .ok_or_else(|| TypeError::UndefinedMethod {
                        struct_name: struct_name.clone(),
                        method: method.clone(),
                        span: expr.span,
                    })?
                    .clone();

                // Method params include 'self' as param 0
                if sig.params.is_empty() {
                    return Err(TypeError::ArgCountMismatch {
                        expected: 1,
                        found: 0,
                        span: expr.span,
                    });
                }

                let self_param_ty = &sig.params[0];
                if let Type::MutReference(_) = self_param_ty {
                    if let ExprKind::Identifier(ref var_name) = target.kind {
                        if let Some(info) = self.lookup_var(var_name) {
                            if !info.is_mut {
                                return Err(TypeError::CannotMutateImmutable {
                                    name: var_name.clone(),
                                    span: target.span,
                                });
                            }
                        }
                    }
                }

                // Check remaining args against params[1..]
                let method_args_expected = sig.params.len() - 1;
                if method_args_expected != args.len() {
                    return Err(TypeError::ArgCountMismatch {
                        expected: method_args_expected,
                        found: args.len(),
                        span: expr.span,
                    });
                }

                for (param_ty, arg_expr) in sig.params[1..].iter().zip(args.iter()) {
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
            ExprKind::StructInit { name, fields } => {
                let s_info = self
                    .structs
                    .get(name)
                    .ok_or_else(|| TypeError::UndefinedStruct {
                        name: name.clone(),
                        span: expr.span,
                    })?
                    .clone();

                let mut provided_fields = HashMap::new();
                for (f_name, f_expr) in fields {
                    let f_ty = self.check_expr(f_expr)?;
                    let expected_ty = s_info
                        .fields
                        .get(f_name)
                        .ok_or_else(|| TypeError::UndefinedField {
                            struct_name: name.clone(),
                            field: f_name.clone(),
                            span: f_expr.span,
                        })?;

                    if !types_compatible(expected_ty, &f_ty) {
                        return Err(TypeError::Mismatch {
                            expected: expected_ty.to_string(),
                            found: f_ty.to_string(),
                            span: f_expr.span,
                        });
                    }

                    provided_fields.insert(f_name.clone(), f_ty);
                }

                for expected_f in &s_info.field_order {
                    if !provided_fields.contains_key(expected_f) {
                        return Err(TypeError::MissingStructField {
                            struct_name: name.clone(),
                            field: expected_f.clone(),
                            span: expr.span,
                        });
                    }
                }

                Ok(Type::Struct(name.clone()))
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

    #[test]
    fn test_struct_typecheck() {
        let code = r#"
संरचना बिन्दु
    क्ष: अंश६४।
    य: अंश६४।
इति

विधान बिन्दु
    क्रिया दूरता(स्व) -> अंश६४
        प्रति स्व.क्ष + स्व.य।
    इति

    क्रिया स्थानान्तरय(चलऋण स्व, अक्ष: अंश६४, अय: अंश६४) -> रिक्त
        स्व.क्ष = स्व.क्ष + अक्ष।
        स्व.य = स्व.य + अय।
    इति
इति

क्रिया मुख्य() -> रिक्त
    मान विकार्य ब = बिन्दु(क्ष: ३.०, य: ४.०)।
    मान द: अंश६४ = ब.दूरता()।
    ब.स्थानान्तरय(१.०, २.०)।
    ब.क्ष = ५.०।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_array_and_indexing_typecheck() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान विकार्य सारणी = [१०, २०, ३०]।
    मान प्रथम: पूर्ण६४ = सारणी[०]।
    सारणी[१] = ५०।
    मान आकार: पूर्ण६४ = सूची_दैर्घ्यम्(सारणी)।
    सूची_संयोजय(सारणी, १००)।
    मान नूतना: सूची = सूची_सृज(१०, ०)।
    मान वाक्य: सूत्र = "नमस्ते"।
    मान दैर्घ्य: पूर्ण६४ = सूत्र_दैर्घ्यम्(वाक्य)।
    मान वर्णः: सूत्र = सूत्र_वर्ण(वाक्य, ०)।
    मान घा: अंश६४ = घाताङ्क(१.०)।
    मान लॉ: अंश६४ = लॉग(२.०)।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_file_io_and_string_utils_typecheck() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    संचिका_लेख("परीक्षण.पाठ", "नमस्ते जगत्")।
    मान अस्ति: द्वैध = संचिका_विद्यते("परीक्षण.पाठ")।
    मान पाठ: सूत्र = संचिका_पठ("परीक्षण.पाठ")।
    मान भागाः: सूची = सूत्र_विभाजय(पाठ, " ")।
    मान संयुक्तम्: सूत्र = सूची_संयोग(भागाः, "-")।
    मान संख्या: अंश६४ = संख्या_पाठ("१२.३४")।
    मान सं_सूत्र: सूत्र = सूत्र_रूप(संख्या)।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut checker = TypeChecker::new();
        assert!(checker.check_program(&program).is_ok());
    }
}

