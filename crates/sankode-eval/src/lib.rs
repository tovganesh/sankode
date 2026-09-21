use sankode_core::{
    format_i64_devanagari, BinaryOp, Expr, ExprKind, FunctionDecl, Program, Statement, TopLevelItem,
    UnaryOp,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}

impl Value {
    pub fn display_devanagari(&self) -> String {
        match self {
            Value::Integer(n) => format_i64_devanagari(*n),
            Value::Float(f) => format!("{:.4}", f),
            Value::String(s) => s.clone(),
            Value::Bool(true) => "सत्यम्".to_string(),
            Value::Bool(false) => "मिथ्या".to_string(),
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
    functions: HashMap<String, FunctionDecl>,
    global_env: Rc<RefCell<Env>>,
    pub stdout_capture: Option<Vec<String>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            global_env: Rc::new(RefCell::new(Env::new())),
            stdout_capture: None,
        }
    }

    pub fn load_program(&mut self, program: &Program) {
        for item in &program.items {
            if let TopLevelItem::Function(func) = item {
                self.functions.insert(func.name.clone(), func.clone());
            }
        }
    }

    pub fn run_main(&mut self) -> Result<Value, RuntimeError> {
        if let Some(main_func) = self.functions.get("मुख्य").cloned() {
            let env = Rc::new(RefCell::new(Env::with_parent(self.global_env.clone())));
            self.execute_function(&main_func, Vec::new(), env)
        } else {
            Err(RuntimeError::MainNotFound)
        }
    }

    fn execute_function(
        &mut self,
        func: &FunctionDecl,
        args: Vec<Value>,
        env: Rc<RefCell<Env>>,
    ) -> Result<Value, RuntimeError> {
        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            env.borrow_mut().define(param.name.clone(), arg, false);
        }

        for stmt in &func.body {
            match self.execute_statement(stmt, env.clone())? {
                Flow::Return(val) => return Ok(val),
                Flow::None => {}
            }
        }

        Ok(Value::Unit)
    }

    fn execute_statement(
        &mut self,
        stmt: &Statement,
        env: Rc<RefCell<Env>>,
    ) -> Result<Flow, RuntimeError> {
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
                }

                if let Some(func) = self.functions.get(callee).cloned() {
                    let func_env = Rc::new(RefCell::new(Env::with_parent(self.global_env.clone())));
                    self.execute_function(&func, evaluated_args, func_env)
                } else {
                    Err(RuntimeError::UndefinedFunction(callee.clone()))
                }
            }
        }
    }

    fn eval_binary_op(&self, op: BinaryOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match (l, r) {
            (Value::Integer(a), Value::Integer(b)) => match op {
                BinaryOp::Add => Ok(Value::Integer(a + b)),
                BinaryOp::Sub => Ok(Value::Integer(a - b)),
                BinaryOp::Mul => Ok(Value::Integer(a * b)),
                BinaryOp::Div => {
                    if b == 0 {
                        Err(RuntimeError::DivisionByZero)
                    } else {
                        Ok(Value::Integer(a / b))
                    }
                }
                BinaryOp::Mod => Ok(Value::Integer(a % b)),
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
}
