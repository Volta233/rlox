use crate::token::*;
use crate::expr::Expr;
use crate::statement::Stmt;
use crate::environment::{RuntimeError, Environment};

type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    globals: Environment,
    environment: Box<Environment>, // 当前作用域
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new(None);
        // 预定义全局函数（如clock）
        let mut env = Environment::new(None);
        env.define("clock".to_string(), Literal::NumberValue(0.0)); // 占位符
        Self {
            globals,
            environment: Box::new(env),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;
        }
        Ok(())
    }

    // 后续实现 execute 和 evaluate 方法
    fn evaluate(&mut self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Variable { name } => self.environment.get(name),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::Minus => self.check_number_operand(operator, &right_val)
                        .map(|n| Literal::NumberValue(-n)),
                    TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(&right_val))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary { left, operator, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                
                match operator.token_type {
                    // 算术运算
                    TokenType::Plus => self.add_values(&left_val, &right_val),
                    TokenType::Minus => self.sub_numbers(&left_val, &right_val, operator),
                    TokenType::Star => self.mul_numbers(&left_val, &right_val, operator),
                    TokenType::Slash => self.div_numbers(&left_val, &right_val, operator),
                    // 比较运算
                    TokenType::Greater => self.compare(&left_val, &right_val, |a,b| a > b),
                    TokenType::GreaterEqual => self.compare(&left_val, &right_val, |a,b| a >= b),
                    TokenType::Less => self.compare(&left_val, &right_val, |a,b| a < b),
                    TokenType::LessEqual => self.compare(&left_val, &right_val, |a,b| a <= b),
                    // 逻辑运算
                    TokenType::EqualEqual => Ok(Literal::Boolean(self.is_equal(&left_val, &right_val))),
                    TokenType::BangEqual => Ok(Literal::Boolean(!self.is_equal(&left_val, &right_val))),
                    _ => Err(RuntimeError::Runtime(operator.clone(), "Invalid operator".into())),
                }
            }
            // 其他表达式类型...
            Expr::Call { callee, paren, arguments } => {
                // 1. 解析被调用对象
                let callee_val = self.evaluate(callee)?;
                
                // 2. 解析参数列表
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }
                
                // 3. 执行调用
                match callee_val {
                    Literal::FunctionValue(func) => self.call_function(&func, args),
                    Literal::ClassValue(cls) => self.call_class_constructor(&cls, args, paren),
                    _ => Err(RuntimeError::Runtime(
                        paren.clone(),
                        "Can only call functions and classes".into()
                    )),
                }
            }
        }
    }

     fn is_truthy(&self, val: &Literal) -> bool {
        match val {
            Literal::Nil => false,
            Literal::Boolean(b) => *b,
            _ => true,
        }
    }

    fn check_number_operand(&self, op: &Token, val: &Literal) -> Result<f64> {
        if let Literal::NumberValue(n) = val {
            Ok(*n)
        } else {
            Err(RuntimeError::Runtime(op.clone(), "Operand must be a number".into()))
        }
    }

    // 实现加法（支持字符串连接）
    fn add_values(&self, a: &Literal, b: &Literal) -> Result<Literal> {
        match (a, b) {
            (Literal::NumberValue(n1), Literal::NumberValue(n2)) => 
                Ok(Literal::NumberValue(n1 + n2)),
            (Literal::StringValue(s1), Literal::StringValue(s2)) =>
                Ok(Literal::StringValue(format!("{}{}", s1, s2))),
            _ => Err(RuntimeError::Runtime(
                Token::new(TokenType::Plus, 0, "+".into(), None),
                "Operands must be two numbers or two strings".into()
            ))
        }
    }

    fn sub_numbers(&self, left: &Literal, right: &Literal, op: &Token) -> Result<Literal> {
        let (a, b) = self.check_number_operands(left, right, op)?;
        Ok(Literal::NumberValue(a - b))
    }
    
    fn mul_numbers(&self, left: &Literal, right: &Literal, op: &Token) -> Result<Literal> {
        let (a, b) = self.check_number_operands(left, right, op)?;
        Ok(Literal::NumberValue(a * b))
    }
    
    fn div_numbers(&self, left: &Literal, right: &Literal, op: &Token) -> Result<Literal> {
        let (a, b) = self.check_number_operands(left, right, op)?;
        if b == 0.0 {
            return Err(RuntimeError::Runtime(op.clone(), "Division by zero".into()));
        }
        Ok(Literal::NumberValue(a / b))
    }
    
    fn is_equal(&self, a: &Literal, b: &Literal) -> bool {
        match (a, b) {
            // Nil只等于Nil
            (Literal::Nil, Literal::Nil) => true,
            
            // 布尔值严格比较
            (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
            
            // 数值比较（需考虑浮点精度问题）
            (Literal::NumberValue(a), Literal::NumberValue(b)) => 
                (a - b).abs() < f64::EPSILON,
            
            // 字符串内容比较
            (Literal::StringValue(a), Literal::StringValue(b)) => a == b,
            
            // 函数比较（指针地址比较）
            (Literal::FunctionValue(a), Literal::FunctionValue(b)) => 
                std::ptr::eq(a, b),
            
            // 类比较（名称和内存地址双重校验）
            (Literal::ClassValue(a), Literal::ClassValue(b)) => 
                a.name == b.name && std::ptr::eq(a, b),
            
            // 其他情况均为不相等
            _ => false
        }
    }

    fn compare<T>(&self, left: &Literal, right: &Literal, comp: T) -> Result<Literal>
    where
        T: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Literal::NumberValue(a), Literal::NumberValue(b)) => {
                Ok(Literal::Boolean(comp(*a, *b)))
            }
            (Literal::StringValue(a), Literal::StringValue(b)) => {
                Ok(Literal::Boolean(comp(a.len() as f64, b.len() as f64)))
            }
            _ => Err(RuntimeError::Runtime(
                Token::new(TokenType::EqualEqual, 0, "".into(), None),
                "Operands must be numbers or strings".into(),
            )),
        }
    }
    
    // 公共类型检查方法
    fn check_number_operands(
        &self,
        left: &Literal,
        right: &Literal,
        op: &Token,
    ) -> Result<(f64, f64)> {
        if let (Literal::NumberValue(a), Literal::NumberValue(b)) = (left, right) {
            Ok((*a, *b))
        } else {
            Err(RuntimeError::Runtime(
                op.clone(),
                "Operands must be numbers".into(),
            ))
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }
            Stmt::Print { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", self.stringify(value));
                Ok(())
            }
            Stmt::VarDecl { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Literal::Nil,
                };
                self.environment.define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block { statements } => {
                let previous = self.environment.clone();
                self.environment = Box::new(Environment::new(Some(previous)));
                let result = self.execute_block(statements);
                if let Some(env) = self.environment.enclosing.take() {
                    self.environment = env;
                }
                result
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_result = self.evaluate(condition)?; // 隔离作用域
                if self.is_truthy(&cond_result) {
                    self.execute(then_branch)
                } else {
                    else_branch.as_ref().map_or(Ok(()), |e| self.execute(e))
                }
            }
            Stmt::While { condition, body } => {
                while {
                    let cond = self.evaluate(condition)?; // 每次循环重新计算条件
                    self.is_truthy(&cond)
                } {
                    self.execute(body)?;
                }
                Ok(())
            }
            // 其他语句处理...
            Stmt::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.execute(init.as_ref())?;
                }
                
                loop {
                    let cond = match condition {
                        Some(c) => self.evaluate(c)?,
                        None => Literal::Boolean(true),
                    };
                    if !self.is_truthy(&cond) {
                        break;
                    }
                    
                    self.execute(body.as_ref())?;
                    
                    if let Some(inc) = increment {
                        self.evaluate(inc)?; 
                    }
                }
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                // 将函数存储为环境中的可调用对象
                let function = LoxFunction {
                    declaration: Box::new(stmt.clone()), // 必须显式装箱
                    closure: Box::new(self.environment.deep_clone())
                };
                self.environment.define(name.lexeme.clone(), Literal::FunctionValue(function));
                Ok(())
            }
            
            Stmt::Class { name, superclass, methods } => {
                // 处理继承关系
                let superclass_literal = match superclass {
                    Some(expr) => self.evaluate(expr)?,
                    None => Literal::None
                };
                
                let superclass = match superclass_literal {
                    Literal::ClassValue(c) => Some(Box::new(c)), 
                    _ => return Err(RuntimeError::Runtime(
                        name.clone(),
                        "Superclass must be a class".into()
                    ))
                };
                
                // 创建类对象
                let class = LoxClass {
                    name: name.lexeme.clone(),
                    methods: methods.iter().map(|m| m.clone()).collect(),
                    superclass
                };
                
                self.environment.define(name.lexeme.clone(), Literal::ClassValue(class));
                Ok(())
            }
            
            Stmt::Return { keyword, value } => {
                let return_value = match value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Literal::Nil,
                };
                // 使用自定义错误类型传递返回值
                Err(RuntimeError::Return(return_value))
            }
        }
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<()> {
        let previous = self.environment.deep_clone(); // 使用深度克隆
        self.environment = Box::new(Environment::new(
            Some(Box::new(previous)) // 直接使用克隆的环境
        ));
        let result = stmts.iter()
                .try_for_each(|stmt| self.execute(stmt));
        if let Some(enclosing) = &self.environment.enclosing {
            self.environment = enclosing.clone();
        } // 恢复环境
        result
    }

    fn stringify(&self, value: Literal) -> String {
        match value {
            Literal::Nil => "nil".into(),
            Literal::Boolean(b) => b.to_string(),
            Literal::NumberValue(n) => format!("{}", n),
            Literal::StringValue(s) => s,
            // 其他类型处理...
            Literal::FunctionValue(f) => format!("<fn {}>", 
                match *f.declaration {
                    Stmt::Function { name, .. } => name.lexeme,
                    _ => "anonymous".into()
                }   
            ),
            Literal::ClassValue(c) => format!("<class {}>", c.name),
            Literal::InstanceValue(i) => format!("<instance of {}>", i.class.name),
            Literal::None => "nil".into(), // 合并None和Nil处理
        }
    }

    fn call_function(&mut self, func: &LoxFunction, args: Vec<Literal>) -> Result<Literal> {
        let (params, body) = match func.declaration.as_ref() {
            Stmt::Function { params, body, .. } => (params, body),
        _ => return Err(RuntimeError::Runtime(
            Token::new( 
                TokenType::Error,
                0,
                "".to_string(),
                None
            ), 
            "Invalid function declaration".into()
        ))
        };
        
        let mut env = Environment::new(
            Some(Box::new((*func.closure).clone())) // 正确克隆闭包
        );
        
        // 参数绑定
        params.iter().zip(args).for_each(|(param, arg)| {
            env.define(param.lexeme.clone(), arg);
        });
        
        // 执行逻辑
        let prev_env = self.environment.clone();
        self.environment = Box::new(env);
        
        let result = self.execute_block(&body);
        self.environment = prev_env;
        result?; // 捕获可能抛出的 Return 错误
        Ok(Literal::Nil) // 或从返回错误中提取值
    }
}