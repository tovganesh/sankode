use sankode_core::{
    format_f64_devanagari, format_i64_devanagari, BinaryOp, Expr, ExprKind, FunctionDecl, Program,
    Statement, StructDecl, TopLevelItem, UnaryOp,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("अज्ञातः चरः (Undefined variable) '{0}'")]
    UndefinedVariable(String),
    #[error("अज्ञाता क्रिया (Undefined function) '{0}'")]
    UndefinedFunction(String),
    #[error("अमान्या क्रिया (Type mismatch / Invalid operation): {0}")]
    TypeMismatch(String),
    #[error("शून्येन भागः अमान्यः (Division by zero)")]
    DivisionByZero,
    #[error("मुख्य-क्रिया न प्राप्ता (No 'मुख्य' function found)")]
    MainNotFound,
    #[error("क्रिया-सीमा अतिक्रान्ता (Execution step limit exceeded): {0} steps (possible infinite loop)")]
    StepLimitExceeded(usize),
    #[error("आह्वान-सीमा अतिक्रान्ता (Call stack depth limit exceeded): {0} frames (infinite recursion)")]
    StackOverflow(usize),
    #[error("अतिप्रवाहः (Integer overflow): {0}")]
    IntegerOverflow(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct {
        name: String,
        fields: Rc<RefCell<HashMap<String, Value>>>,
    },
    List(Rc<RefCell<Vec<Value>>>),
    Unit,
}

impl Value {
    pub fn display_devanagari(&self) -> String {
        match self {
            Value::Integer(n) => format_i64_devanagari(*n),
            Value::Float(f) => format_f64_devanagari(*f),
            Value::String(s) => s.clone(),
            Value::Bool(true) => "सत्यम्".to_string(),
            Value::Bool(false) => "मिथ्या".to_string(),
            Value::Struct { name, fields } => {
                let mut parts = Vec::new();
                for (k, v) in fields.borrow().iter() {
                    parts.push(format!("{}: {}", k, v.display_devanagari()));
                }
                format!("{}({})", name, parts.join(", "))
            }
            Value::List(list) => {
                let parts: Vec<String> = list.borrow().iter().map(|v| v.display_devanagari()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Unit => "रिक्त".to_string(),
        }
    }
}

pub struct Env {
    variables: HashMap<String, Value>,
    mutability: HashMap<String, bool>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            mutability: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Env>>) -> Self {
        Self {
            variables: HashMap::new(),
            mutability: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, val: Value, is_mut: bool) {
        self.variables.insert(name.clone(), val);
        self.mutability.insert(name, is_mut);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.variables.get(name) {
            Some(val.clone())
        } else if let Some(ref parent) = self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn is_mut(&self, name: &str) -> Option<bool> {
        if let Some(m) = self.mutability.get(name) {
            Some(*m)
        } else if let Some(ref parent) = self.parent {
            parent.borrow().is_mut(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), RuntimeError> {
        if self.variables.contains_key(name) {
            let is_mut = *self.mutability.get(name).unwrap_or(&false);
            if !is_mut {
                return Err(RuntimeError::TypeMismatch(format!(
                    "चरः '{}' अविकार्यः अस्ति (Cannot assign to immutable variable)",
                    name
                )));
            }
            self.variables.insert(name.to_string(), val);
            Ok(())
        } else if let Some(ref parent) = self.parent {
            parent.borrow_mut().assign(name, val)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }
}

pub enum Flow {
    None,
    Return(Value),
}

pub struct Interpreter {
    pub functions: HashMap<String, FunctionDecl>,
    pub methods: HashMap<(String, String), FunctionDecl>,
    pub structs: HashMap<String, StructDecl>,
    pub global_env: Rc<RefCell<Env>>,
    pub top_level_statements: Vec<Statement>,
    pub stdout_capture: Option<Vec<String>>,
    pub step_count: usize,
    pub max_steps: Option<usize>,
    pub call_depth: usize,
    pub max_call_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            methods: HashMap::new(),
            structs: HashMap::new(),
            global_env: Rc::new(RefCell::new(Env::new())),
            top_level_statements: Vec::new(),
            stdout_capture: None,
            step_count: 0,
            max_steps: Some(10_000_000),
            call_depth: 0,
            max_call_depth: 500,
        }
    }

    pub fn load_program(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevelItem::Function(func) => {
                    self.functions.insert(func.name.clone(), func.clone());
                }
                TopLevelItem::Struct(s) => {
                    self.structs.insert(s.name.clone(), s.clone());
                }
                TopLevelItem::Impl(imp) => {
                    for method in &imp.methods {
                        self.methods.insert(
                            (imp.target.clone(), method.name.clone()),
                            method.clone(),
                        );
                    }
                }
                TopLevelItem::Statement(stmt) => {
                    self.top_level_statements.push(stmt.clone());
                }
                TopLevelItem::Comment(_) => {}
            }
        }
    }

    pub fn run_main(&mut self) -> Result<Value, RuntimeError> {
        // Execute top-level script statements first
        for stmt in &self.top_level_statements.clone() {
            if let Flow::Return(val) = self.execute_statement(stmt, self.global_env.clone())? {
                return Ok(val);
            }
        }

        // Then execute मुख्य if it exists
        if let Some(main_func) = self.functions.get("मुख्य").cloned() {
            let env = Rc::new(RefCell::new(Env::with_parent(self.global_env.clone())));
            self.execute_function(&main_func, Vec::new(), env)
        } else if !self.top_level_statements.is_empty() {
            Ok(Value::Unit)
        } else {
            Err(RuntimeError::MainNotFound)
        }
    }

    pub fn eval_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, RuntimeError> {
        match stmt {
            Statement::Expr(expr) => {
                let val = self.eval_expr(expr, self.global_env.clone())?;
                Ok(Some(val))
            }
            _ => {
                self.execute_statement(stmt, self.global_env.clone())?;
                Ok(None)
            }
        }
    }

    fn execute_function(
        &mut self,
        func: &FunctionDecl,
        args: Vec<Value>,
        env: Rc<RefCell<Env>>,
    ) -> Result<Value, RuntimeError> {
        self.call_depth += 1;
        if self.call_depth > self.max_call_depth {
            self.call_depth -= 1;
            return Err(RuntimeError::StackOverflow(self.max_call_depth));
        }

        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            let is_mut = param.name == "स्व";
            env.borrow_mut().define(param.name.clone(), arg, is_mut);
        }

        for stmt in &func.body {
            match self.execute_statement(stmt, env.clone()) {
                Ok(Flow::Return(val)) => {
                    self.call_depth -= 1;
                    return Ok(val);
                }
                Ok(Flow::None) => {}
                Err(e) => {
                    self.call_depth -= 1;
                    return Err(e);
                }
            }
        }

        self.call_depth -= 1;
        Ok(Value::Unit)
    }

    fn execute_statement(
        &mut self,
        stmt: &Statement,
        env: Rc<RefCell<Env>>,
    ) -> Result<Flow, RuntimeError> {
        self.step_count += 1;
        if let Some(max) = self.max_steps {
            if self.step_count > max {
                return Err(RuntimeError::StepLimitExceeded(max));
            }
        }

        match stmt {
            Statement::VarDecl {
                name,
                is_mut,
                init,
                ..
            } => {
                let val = self.eval_expr(init, env.clone())?;
                env.borrow_mut().define(name.clone(), val, *is_mut);
                Ok(Flow::None)
            }
            Statement::Assignment { target, value, .. } => {
                let val = self.eval_expr(value, env.clone())?;
                env.borrow_mut().assign(target, val)?;
                Ok(Flow::None)
            }
            Statement::FieldAssignment {
                target,
                field,
                value,
                ..
            } => {
                let env_ref = env.borrow();
                let is_mut = env_ref.is_mut(target).unwrap_or(false);
                if !is_mut {
                    return Err(RuntimeError::TypeMismatch(format!(
                        "चरः '{}' अविकार्यः अस्ति (Cannot assign to immutable variable)",
                        target
                    )));
                }
                let target_val = env_ref.get(target).ok_or_else(|| {
                    RuntimeError::UndefinedVariable(target.clone())
                })?;
                drop(env_ref);

                let val = self.eval_expr(value, env)?;
                match target_val {
                    Value::Struct { fields, .. } => {
                        fields.borrow_mut().insert(field.clone(), val);
                        Ok(Flow::None)
                    }
                    _ => Err(RuntimeError::TypeMismatch(format!(
                        "चरः '{}' संरचना नास्ति",
                        target
                    ))),
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_val = self.eval_expr(condition, env.clone())?;
                let is_truthy = match cond_val {
                    Value::Bool(b) => b,
                    Value::Integer(n) => n != 0,
                    _ => false,
                };

                let block_env = Rc::new(RefCell::new(Env::with_parent(env.clone())));
                if is_truthy {
                    for s in then_branch {
                        if let Flow::Return(v) = self.execute_statement(s, block_env.clone())? {
                            return Ok(Flow::Return(v));
                        }
                    }
                } else if let Some(ref else_stmts) = else_branch {
                    for s in else_stmts {
                        if let Flow::Return(v) = self.execute_statement(s, block_env.clone())? {
                            return Ok(Flow::Return(v));
                        }
                    }
                }
                Ok(Flow::None)
            }
            Statement::While { condition, body, .. } => {
                loop {
                    let cond_val = self.eval_expr(condition, env.clone())?;
                    let is_truthy = match cond_val {
                        Value::Bool(b) => b,
                        Value::Integer(n) => n != 0,
                        _ => false,
                    };
                    if !is_truthy {
                        break;
                    }

                    let block_env = Rc::new(RefCell::new(Env::with_parent(env.clone())));
                    for s in body {
                        if let Flow::Return(v) = self.execute_statement(s, block_env.clone())? {
                            return Ok(Flow::Return(v));
                        }
                    }
                }
                Ok(Flow::None)
            }
            Statement::Return { value, .. } => {
                let ret_val = if let Some(ref expr) = value {
                    self.eval_expr(expr, env)?
                } else {
                    Value::Unit
                };
                Ok(Flow::Return(ret_val))
            }
            Statement::IndexAssignment {
                target,
                index,
                value,
                ..
            } => {
                let target_val = self.eval_expr(target, env.clone())?;
                let idx_val = self.eval_expr(index, env.clone())?;
                let val = self.eval_expr(value, env)?;

                let idx = match idx_val {
                    Value::Integer(i) => {
                        if i < 0 {
                            return Err(RuntimeError::TypeMismatch("ऋणात्मकः सूचकः अमान्यः (Negative index invalid)".to_string()));
                        }
                        i as usize
                    }
                    _ => return Err(RuntimeError::TypeMismatch("सूचकः पूर्ण६४ भवेत् (Index must be integer)".to_string())),
                };

                match target_val {
                    Value::List(list) => {
                        let mut borrowed = list.borrow_mut();
                        if idx >= borrowed.len() {
                            return Err(RuntimeError::TypeMismatch(format!(
                                "सूचकातिक्रमः (Index out of bounds): सूचकः {}, आकारः {}",
                                idx, borrowed.len()
                            )));
                        }
                        borrowed[idx] = val;
                        Ok(Flow::None)
                    }
                    _ => Err(RuntimeError::TypeMismatch("लक्ष्यं सूची नास्ति (Target is not a list)".to_string())),
                }
            }
            Statement::Expr(expr) => {
                self.eval_expr(expr, env)?;
                Ok(Flow::None)
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Env>>) -> Result<Value, RuntimeError> {
        match &expr.kind {
            ExprKind::DevanagariInteger(val, _) => Ok(Value::Integer(*val)),
            ExprKind::DevanagariFloat(val, _) => Ok(Value::Float(*val)),
            ExprKind::StringLiteral(s) => Ok(Value::String(s.clone())),
            ExprKind::BoolLiteral(b) => Ok(Value::Bool(*b)),
            ExprKind::Identifier(id) => {
                env.borrow().get(id).ok_or_else(|| RuntimeError::UndefinedVariable(id.clone()))
            }
            ExprKind::Unary { op, expr } => {
                let val = self.eval_expr(expr, env)?;
                match (op, val) {
                    (UnaryOp::Neg, Value::Integer(n)) => Ok(Value::Integer(-n)),
                    (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err(RuntimeError::TypeMismatch("अमान्या एकपदीय-क्रिया".to_string())),
                }
            }
            ExprKind::Borrow { expr, .. } => {
                // In early tree-walk runtime, passes by value/reference identically
                self.eval_expr(expr, env)
            }
            ExprKind::Binary { left, op, right } => {
                let l = self.eval_expr(left, env.clone())?;
                let r = self.eval_expr(right, env)?;
                self.eval_binary_op(*op, l, r)
            }
            ExprKind::Call { callee, args } => {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, env.clone())?);
                }

                // Built-in print: मुद्रय(...)
                if callee == "मुद्रय" {
                    let mut output_parts = Vec::new();
                    for a in &evaluated_args {
                        output_parts.push(a.display_devanagari());
                    }
                    let line = output_parts.join("");
                    if let Some(ref mut capture) = self.stdout_capture {
                        capture.push(line);
                    } else {
                        println!("{}", line);
                    }
                    return Ok(Value::Unit);
                } else if callee == "सूची_सृज" {
                    if evaluated_args.len() != 2 {
                        return Err(RuntimeError::TypeMismatch("सूची_सृज द्वौ तर्कौ अपेक्षते (आकार, प्रारम्भिक_मान)".to_string()));
                    }
                    let size = match evaluated_args[0] {
                        Value::Integer(n) if n >= 0 => n as usize,
                        _ => return Err(RuntimeError::TypeMismatch("आकारः पूर्ण६४ भवेत्".to_string())),
                    };
                    let initial_val = evaluated_args[1].clone();
                    let list = vec![initial_val; size];
                    return Ok(Value::List(Rc::new(RefCell::new(list))));
                } else if callee == "सूची_दैर्घ्यम्" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("सूची_दैर्घ्यम् एकं तर्कम् अपेक्षते".to_string()));
                    }
                    match &evaluated_args[0] {
                        Value::List(list) => return Ok(Value::Integer(list.borrow().len() as i64)),
                        _ => return Err(RuntimeError::TypeMismatch("सूची अपेक्षिता".to_string())),
                    }
                } else if callee == "सूची_संयोजय" {
                    if evaluated_args.len() != 2 {
                        return Err(RuntimeError::TypeMismatch("सूची_संयोजय द्वौ तर्कौ अपेक्षते (सूची, मान)".to_string()));
                    }
                    match &evaluated_args[0] {
                        Value::List(list) => {
                            list.borrow_mut().push(evaluated_args[1].clone());
                            return Ok(Value::Unit);
                        }
                        _ => return Err(RuntimeError::TypeMismatch("सूची अपेक्षिता".to_string())),
                    }
                } else if callee == "सूत्र_दैर्घ्यम्" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("सूत्र_दैर्घ्यम् एकं तर्कम् अपेक्षते".to_string()));
                    }
                    match &evaluated_args[0] {
                        Value::String(s) => return Ok(Value::Integer(s.chars().count() as i64)),
                        _ => return Err(RuntimeError::TypeMismatch("सूत्रम् अपेक्षितम्".to_string())),
                    }
                } else if callee == "सूत्र_वर्ण" {
                    if evaluated_args.len() != 2 {
                        return Err(RuntimeError::TypeMismatch("सूत्र_वर्ण द्वौ तर्कौ अपेक्षते (सूत्र, सूचक)".to_string()));
                    }
                    match (&evaluated_args[0], &evaluated_args[1]) {
                        (Value::String(s), Value::Integer(idx)) => {
                            let idx = *idx;
                            if idx < 0 {
                                return Err(RuntimeError::TypeMismatch("ऋणात्मकः सूचकः अमान्यः".to_string()));
                            }
                            let ch = s.chars().nth(idx as usize).ok_or_else(|| {
                                RuntimeError::TypeMismatch(format!("सूचकातिक्रमः (Index out of bounds): {}", idx))
                            })?;
                            return Ok(Value::String(ch.to_string()));
                        }
                        _ => return Err(RuntimeError::TypeMismatch("सूत्रं पूर्ण६४ च अपेक्षितौ".to_string())),
                    }
                } else if callee == "सूत्र_अंश" {
                    if evaluated_args.len() != 3 {
                        return Err(RuntimeError::TypeMismatch("सूत्र_अंश त्रीन् तर्कान् अपेक्षते (सूत्र, आरम्भ, समाप्ति)".to_string()));
                    }
                    match (&evaluated_args[0], &evaluated_args[1], &evaluated_args[2]) {
                        (Value::String(s), Value::Integer(start), Value::Integer(end)) => {
                            let start = (*start).max(0) as usize;
                            let end = (*end).max(0) as usize;
                            let sub: String = s.chars().skip(start).take(end.saturating_sub(start)).collect();
                            return Ok(Value::String(sub));
                        }
                        _ => return Err(RuntimeError::TypeMismatch("सूत्रं पूर्ण६४ च अपेक्षितौ".to_string())),
                    }
                } else if callee == "लॉग" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("लॉग एकं तर्कम् अपेक्षते".to_string()));
                    }
                    let num = match evaluated_args[0] {
                        Value::Float(f) => f,
                        Value::Integer(n) => n as f64,
                        _ => return Err(RuntimeError::TypeMismatch("संख्या अपेक्षिता".to_string())),
                    };
                    return Ok(Value::Float(num.ln()));
                } else if callee == "घाताङ्क" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("घाताङ्क एकं तर्कम् अपेक्षते".to_string()));
                    }
                    let num = match evaluated_args[0] {
                        Value::Float(f) => f,
                        Value::Integer(n) => n as f64,
                        _ => return Err(RuntimeError::TypeMismatch("संख्या अपेक्षिता".to_string())),
                    };
                    return Ok(Value::Float(num.exp()));
                } else if callee == "वर्गमूल" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("वर्गमूल एकं तर्कम् अपेक्षते".to_string()));
                    }
                    let num = match evaluated_args[0] {
                        Value::Float(f) => f,
                        Value::Integer(n) => n as f64,
                        _ => return Err(RuntimeError::TypeMismatch("संख्या अपेक्षिता".to_string())),
                    };
                    return Ok(Value::Float(num.sqrt()));
                } else if callee == "पूर्णाङ्क" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("पूर्णाङ्क एकं तर्कम् अपेक्षते".to_string()));
                    }
                    match evaluated_args[0] {
                        Value::Float(f) => return Ok(Value::Integer(f as i64)),
                        Value::Integer(n) => return Ok(Value::Integer(n)),
                        _ => return Err(RuntimeError::TypeMismatch("संख्या अपेक्षिता".to_string())),
                    }
                } else if callee == "अंशाङ्क" {
                    if evaluated_args.len() != 1 {
                        return Err(RuntimeError::TypeMismatch("अंशाङ्क एकं तर्कम् अपेक्षते".to_string()));
                    }
                    match evaluated_args[0] {
                        Value::Integer(n) => return Ok(Value::Float(n as f64)),
                        Value::Float(f) => return Ok(Value::Float(f)),
                        _ => return Err(RuntimeError::TypeMismatch("संख्या अपेक्षिता".to_string())),
                    }
                }

                if let Some(func) = self.functions.get(callee).cloned() {
                    let func_env = Rc::new(RefCell::new(Env::with_parent(self.global_env.clone())));
                    self.execute_function(&func, evaluated_args, func_env)
                } else {
                    Err(RuntimeError::UndefinedFunction(callee.clone()))
                }
            }
            ExprKind::FieldAccess { target, field } => {
                let target_val = self.eval_expr(target, env)?;
                match target_val {
                    Value::Struct { ref fields, .. } => fields
                        .borrow()
                        .get(field)
                        .cloned()
                        .ok_or_else(|| RuntimeError::UndefinedVariable(field.clone())),
                    _ => Err(RuntimeError::TypeMismatch(
                        "संरचनायाः क्षेत्रं न लब्धम् (Expected struct for field access)".to_string(),
                    )),
                }
            }
            ExprKind::MethodCall {
                target,
                method,
                args,
            } => {
                let target_val = self.eval_expr(target, env.clone())?;
                let struct_name = match &target_val {
                    Value::Struct { name, .. } => name.clone(),
                    _ => {
                        return Err(RuntimeError::TypeMismatch(
                            "विधानक्रिया संरचनायाः उपरि एव प्रयुज्यते (Method call requires struct target)".to_string(),
                        ));
                    }
                };

                let method_decl = self
                    .methods
                    .get(&(struct_name.clone(), method.clone()))
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::UndefinedFunction(format!("{}.{}", struct_name, method))
                    })?;

                let mut evaluated_args = Vec::new();
                // First parameter is self ('स्व')
                evaluated_args.push(target_val);
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, env.clone())?);
                }

                let method_env = Rc::new(RefCell::new(Env::with_parent(self.global_env.clone())));
                self.execute_function(&method_decl, evaluated_args, method_env)
            }
            ExprKind::ArrayLiteral(elements) => {
                let mut list = Vec::new();
                for elem in elements {
                    list.push(self.eval_expr(elem, env.clone())?);
                }
                Ok(Value::List(Rc::new(RefCell::new(list))))
            }
            ExprKind::Index { target, index } => {
                let target_val = self.eval_expr(target, env.clone())?;
                let idx_val = self.eval_expr(index, env)?;

                let idx = match idx_val {
                    Value::Integer(i) => {
                        if i < 0 {
                            return Err(RuntimeError::TypeMismatch("ऋणात्मकः सूचकः अमान्यः (Negative index invalid)".to_string()));
                        }
                        i as usize
                    }
                    _ => return Err(RuntimeError::TypeMismatch("सूचकः पूर्ण६४ भवेत् (Index must be integer)".to_string())),
                };

                match target_val {
                    Value::List(list) => {
                        let borrowed = list.borrow();
                        borrowed.get(idx).cloned().ok_or_else(|| {
                            RuntimeError::TypeMismatch(format!(
                                "सूचकातिक्रमः (Index out of bounds): सूचकः {}, आकारः {}",
                                idx, borrowed.len()
                            ))
                        })
                    }
                    Value::String(s) => {
                        let ch = s.chars().nth(idx).ok_or_else(|| {
                            RuntimeError::TypeMismatch(format!(
                                "सूचकातिक्रमः (Index out of bounds): सूचकः {}",
                                idx
                            ))
                        })?;
                        Ok(Value::String(ch.to_string()))
                    }
                    _ => Err(RuntimeError::TypeMismatch("सूची वा सूत्रम् अपेक्षितम् (Expected list or string)".to_string())),
                }
            }
            ExprKind::StructInit { name, fields } => {
                let mut field_values = HashMap::new();
                for (f_name, f_expr) in fields {
                    let f_val = self.eval_expr(f_expr, env.clone())?;
                    field_values.insert(f_name.clone(), f_val);
                }
                Ok(Value::Struct {
                    name: name.clone(),
                    fields: Rc::new(RefCell::new(field_values)),
                })
            }
        }
    }

    fn eval_binary_op(&self, op: BinaryOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match (l, r) {
            (Value::Integer(a), Value::Integer(b)) => match op {
                BinaryOp::Add => a.checked_add(b).map(Value::Integer).ok_or_else(|| {
                    RuntimeError::IntegerOverflow("योगे अतिप्रवाहः (Integer overflow in addition)".to_string())
                }),
                BinaryOp::Sub => a.checked_sub(b).map(Value::Integer).ok_or_else(|| {
                    RuntimeError::IntegerOverflow("व्यवकलने अतिप्रवाहः (Integer overflow in subtraction)".to_string())
                }),
                BinaryOp::Mul => a.checked_mul(b).map(Value::Integer).ok_or_else(|| {
                    RuntimeError::IntegerOverflow("गुणने अतिप्रवाहः (Integer overflow in multiplication)".to_string())
                }),
                BinaryOp::Div => {
                    if b == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        a.checked_div(b).map(Value::Integer).ok_or_else(|| {
                            RuntimeError::IntegerOverflow("भागहारे अतिप्रवाहः (Integer overflow in division)".to_string())
                        })
                    }
                }
                BinaryOp::Mod => {
                    if b == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        a.checked_rem(b).map(Value::Integer).ok_or_else(|| {
                            RuntimeError::IntegerOverflow("शेषे अतिप्रवाहः (Integer overflow in modulo)".to_string())
                        })
                    }
                }
                BinaryOp::Equal => Ok(Value::Bool(a == b)),
                BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                BinaryOp::Less => Ok(Value::Bool(a < b)),
                BinaryOp::LessEq => Ok(Value::Bool(a <= b)),
                BinaryOp::Greater => Ok(Value::Bool(a > b)),
                BinaryOp::GreaterEq => Ok(Value::Bool(a >= b)),
            },
            (Value::Float(a), Value::Float(b)) => match op {
                BinaryOp::Add => Ok(Value::Float(a + b)),
                BinaryOp::Sub => Ok(Value::Float(a - b)),
                BinaryOp::Mul => Ok(Value::Float(a * b)),
                BinaryOp::Div => Ok(Value::Float(a / b)),
                BinaryOp::Equal => Ok(Value::Bool(a == b)),
                BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                BinaryOp::Less => Ok(Value::Bool(a < b)),
                BinaryOp::LessEq => Ok(Value::Bool(a <= b)),
                BinaryOp::Greater => Ok(Value::Bool(a > b)),
                BinaryOp::GreaterEq => Ok(Value::Bool(a >= b)),
                _ => Err(RuntimeError::TypeMismatch("अमान्या क्रिया".to_string())),
            },
            (Value::String(a), Value::String(b)) => match op {
                BinaryOp::Add => Ok(Value::String(format!("{}{}", a, b))),
                BinaryOp::Equal => Ok(Value::Bool(a == b)),
                BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                _ => Err(RuntimeError::TypeMismatch("अमान्या क्रिया".to_string())),
            },
            (Value::Bool(a), Value::Bool(b)) => match op {
                BinaryOp::Equal => Ok(Value::Bool(a == b)),
                BinaryOp::NotEqual => Ok(Value::Bool(a != b)),
                _ => Err(RuntimeError::TypeMismatch("अमान्या क्रिया".to_string())),
            },
            _ => Err(RuntimeError::TypeMismatch("अमान्या क्रिया".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sankode_lexer::Lexer;
    use sankode_parser::Parser;

    #[test]
    fn test_execute_hello_world() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मुद्रय("नमस्ते जगत्!")।
    मान गणना = १०।
    मुद्रय("गणना = ", गणना)।
इति
"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let mut interp = Interpreter::new();
        interp.stdout_capture = Some(Vec::new());
        interp.load_program(&program);
        interp.run_main().unwrap();

        let stdout = interp.stdout_capture.unwrap();
        assert_eq!(stdout[0], "नमस्ते जगत्!");
        assert_eq!(stdout[1], "गणना = १०");
    }

    #[test]
    fn test_fibonacci() {
        let code = r#"
क्रिया फिबोनाची(संख्या: पूर्ण६४) -> पूर्ण६४
    यदि संख्या <= १
        प्रति संख्या।
    इति
    प्रति फिबोनाची(संख्या - १) + फिबोनाची(संख्या - २)।
इति

क्रिया मुख्य() -> रिक्त
    मान परिणाम = फिबोनाची(७)।
    मुद्रय("परिणाम = ", परिणाम)।
इति
"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let mut interp = Interpreter::new();
        interp.stdout_capture = Some(Vec::new());
        interp.load_program(&program);
        interp.run_main().unwrap();

        let stdout = interp.stdout_capture.unwrap();
        assert_eq!(stdout[0], "परिणाम = १३");
    }

    #[test]
    fn test_struct_methods_execution() {
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
    मान द = ब.दूरता()।
    मुद्रय("दूरता = ", द)।
    ब.स्थानान्तरय(१.०, २.०)।
    मान द२ = ब.दूरता()।
    मुद्रय("नूतन-दूरता = ", द२)।
    ब.क्ष = १०.०।
    मुद्रय("अन्तिम-क्ष = ", ब.क्ष)।
इति
"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        let mut interp = Interpreter::new();
        interp.stdout_capture = Some(Vec::new());
        interp.load_program(&program);
        interp.run_main().unwrap();

        let stdout = interp.stdout_capture.unwrap();
        assert_eq!(stdout[0], "दूरता = ७.००००");
        assert_eq!(stdout[1], "नूतन-दूरता = १०.००००");
        assert_eq!(stdout[2], "अन्तिम-क्ष = १०.००००");
    }

    #[test]
    fn test_infinite_loop_protection() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    यावत् सत्यम्
        मान क = १।
    इति
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();

        let mut interp = Interpreter::new();
        interp.max_steps = Some(100);
        interp.load_program(&program);
        let err = interp.run_main().unwrap_err();
        match err {
            RuntimeError::StepLimitExceeded(limit) => {
                assert_eq!(limit, 100);
            }
            _ => panic!("Expected StepLimitExceeded error"),
        }
    }

    #[test]
    fn test_recursion_limit_protection() {
        let code = r#"
क्रिया अनन्त_आह्वानम्() -> रिक्त
    अनन्त_आह्वानम्()।
इति

क्रिया मुख्य() -> रिक्त
    अनन्त_आह्वानम्()।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();

        let mut interp = Interpreter::new();
        interp.max_call_depth = 20;
        interp.load_program(&program);
        let err = interp.run_main().unwrap_err();
        match err {
            RuntimeError::StackOverflow(depth) => {
                assert_eq!(depth, 20);
            }
            _ => panic!("Expected StackOverflow error"),
        }
    }

    #[test]
    fn test_array_and_math_execution() {
        let code = r#"
क्रिया मुख्य() -> रिक्त
    मान विकार्य सारणी = [१०, २०, ३०]।
    मुद्रय("प्रारम्भिकः = ", सारणी[०])।
    सारणी[०] = ९९।
    मुद्रय("परिवर्तितः = ", सारणी[०])।
    मान आकार = सूची_दैर्घ्यम्(सारणी)।
    मुद्रय("आकारः = ", आकार)।
    सूची_संयोजय(सारणी, ४०)।
    मुद्रय("नूतनाकारः = ", सूची_दैर्घ्यम्(सारणी))।

    मान वाक्य = "शकुन्तला"।
    मुद्रय("वर्णसङ्ख्या = ", सूत्र_दैर्घ्यम्(वाक्य))।
    मुद्रय("प्रथमवर्णः = ", सूत्र_वर्ण(वाक्य, ०))।
    मुद्रय("अंशः = ", सूत्र_अंश(वाक्य, ०, ५))।

    मान घात = घाताङ्क(०.०)।
    मुद्रय("घात = ", घात)।
    मान मूल = वर्गमूल(१६.०)।
    मुद्रय("मूल = ", मूल)।
इति
"#;
        let tokens = Lexer::new(code).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();

        let mut interp = Interpreter::new();
        interp.stdout_capture = Some(Vec::new());
        interp.load_program(&program);
        assert!(interp.run_main().is_ok());

        let stdout = interp.stdout_capture.unwrap();
        assert_eq!(stdout[0], "प्रारम्भिकः = १०");
        assert_eq!(stdout[1], "परिवर्तितः = ९९");
        assert_eq!(stdout[2], "आकारः = ३");
        assert_eq!(stdout[3], "नूतनाकारः = ४");
        assert_eq!(stdout[4], "वर्णसङ्ख्या = ८");
        assert_eq!(stdout[5], "प्रथमवर्णः = श");
        assert_eq!(stdout[6], "अंशः = शकुन्");
        assert_eq!(stdout[7], "घात = १.००००");
        assert_eq!(stdout[8], "मूल = ४.००००");
    }
}


