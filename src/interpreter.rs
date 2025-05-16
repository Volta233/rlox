use crate::environment::{Environment, RuntimeError};
use crate::expr::Expr;
use crate::statement::Stmt;
use crate::token::*;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Box<Environment>, // 当前作用域
}

impl Interpreter {
    fn get_call_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Get { name, .. } => name.lexeme.clone(),
            _ => String::new(),
        }
    }
    fn evaluate_args(&mut self, exprs: &[Expr]) -> Result<Vec<Literal>> {
        // 移除显式错误类型
        exprs.iter().map(|expr| self.evaluate(expr)).collect()
    }

    pub fn new() -> Self {
        // 预定义全局函数（如clock）
        let mut env = Environment::new(None);
        env.define("clock".to_string(), Literal::NumberValue(0.0)); // 占位符
        Self {
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
                    TokenType::Minus => self
                        .check_number_operand(operator, &right_val)
                        .map(|n| Literal::NumberValue(-n)),
                    TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(&right_val))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;

                match operator.token_type {
                    // 算术运算
                    TokenType::Plus => self.add_values(&left_val, &right_val),
                    TokenType::Minus => self.sub_numbers(&left_val, &right_val, operator),
                    TokenType::Star => self.mul_numbers(&left_val, &right_val, operator),
                    TokenType::Slash => self.div_numbers(&left_val, &right_val, operator),
                    // 比较运算
                    TokenType::Greater => self.compare(&left_val, &right_val, |a, b| a > b),
                    TokenType::GreaterEqual => self.compare(&left_val, &right_val, |a, b| a >= b),
                    TokenType::Less => self.compare(&left_val, &right_val, |a, b| a < b),
                    TokenType::LessEqual => self.compare(&left_val, &right_val, |a, b| a <= b),
                    // 逻辑运算
                    TokenType::EqualEqual => {
                        Ok(Literal::Boolean(self.is_equal(&left_val, &right_val)))
                    }
                    TokenType::BangEqual => {
                        Ok(Literal::Boolean(!self.is_equal(&left_val, &right_val)))
                    }
                    TokenType::And => self.logical_and(&left_val, &right_val, operator),
                    TokenType::Or => self.logical_or(&left_val, &right_val, operator),
                    _ => Err(RuntimeError::Runtime(
                        operator.clone(),
                        "Invalid operator".into(),
                    )),
                }
            }
            // 其他表达式类型...
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee_val = self.evaluate(callee)?;
                let args = self.evaluate_args(arguments)?;

                match callee_val {
                    Literal::FunctionValue(func) => self.call_function(&func, args, paren),
                    Literal::ClassValue(cls) => {
                        // 类实例化调用
                        let instance = self.call_class_constructor(&cls, args, paren)?;

                        // 使用引用模式匹配
                        if let Literal::InstanceValue(ref inst) = instance {
                            if let Some(init) = inst.class.find_method("init") {
                                // 传递实例引用
                                self.call_method(inst, init, vec![], paren)?;
                            }
                        }

                        Ok(instance)
                    }
                    // 处理实例方法调用
                    Literal::InstanceValue(inst) => {
                        // 通过实例的类查找方法
                        if let Some(method) = inst.class.find_method(&self.get_call_name(callee)) {
                            self.call_method(&inst, method, args, paren)
                        } else {
                            Err(RuntimeError::Runtime(
                                paren.clone(),
                                "Undefined method".into(),
                            ))
                        }
                    }
                    _ => Err(RuntimeError::Runtime(
                        paren.clone(),
                        "Can only call functions and classes".into(),
                    )),
                }
            }
            Expr::Super { keyword, method } => {
                // 步骤1：获取超类引用
                let super_class = match self.environment.get(keyword)? {
                    Literal::ClassValue(c) => c,
                    _ => {
                        return Err(RuntimeError::Runtime(
                            keyword.clone(),
                            "Invalid super class".into(),
                        ));
                    }
                };

                // 步骤2：获取当前实例的this绑定
                let this_instance = match self.environment.get(&Token::this())? {
                    Literal::InstanceValue(i) => i,
                    _ => {
                        return Err(RuntimeError::Runtime(
                            keyword.clone(),
                            "super must be used in instance method".into(),
                        ));
                    }
                };

                // 步骤3：查找超类方法
                let method = super_class.find_method(&method.lexeme).ok_or_else(|| {
                    RuntimeError::Runtime(
                        method.clone(),
                        format!("Undefined method '{}'", method.lexeme),
                    )
                })?;

                // 步骤4：创建闭包环境（绑定this）
                if let Literal::FunctionValue(mut func) = method {
                    // 克隆原闭包环境
                    let mut closure = (*func.closure).clone();

                    // 注入 this 实例
                    closure.define("this".into(), Literal::InstanceValue(this_instance));
                    func.closure = Box::new(closure);

                    Ok(Literal::FunctionValue(func))
                } else {
                    // 使用调用方法时的方法名 Token 来构建错误
                    Err(RuntimeError::Runtime(
                        keyword.clone(),
                        format!("'{}' is not a function", keyword.lexeme).into(),
                    ))
                }
            }
            Expr::Get { object, name } => {
                let obj = self.evaluate(object)?;
                if let Literal::InstanceValue(instance) = obj {
                    instance.fields.get(&name.lexeme).cloned().ok_or_else(|| {
                        RuntimeError::Runtime(
                            name.clone(),
                            format!("Undefined property '{}'", name.lexeme),
                        )
                    })
                } else {
                    Err(RuntimeError::Runtime(
                        name.clone(),
                        "Only instances have properties".into(),
                    ))
                }
            }
            // 变量赋值表达式
            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.environment.assign(name, val.clone())?;
                Ok(val)
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let obj = self.evaluate(object)?;
                let val = self.evaluate(value)?;

                if let Literal::InstanceValue(mut instance) = obj {
                    instance.fields.insert(name.lexeme.clone(), val);
                    Ok(Literal::InstanceValue(instance))
                } else {
                    Err(RuntimeError::Runtime(
                        name.clone(),
                        "Only instances can have fields".into(),
                    ))
                }
            }
            Expr::This { keyword } => {
                // 从当前环境获取this绑定
                let this_value = self.environment.get(keyword)?;

                // 验证必须是实例类型
                if let Literal::InstanceValue(instance) = this_value {
                    Ok(Literal::InstanceValue(instance))
                } else {
                    Err(RuntimeError::Runtime(
                        keyword.clone(),
                        "Invalid 'this' context".into(),
                    ))
                }
            }
        }
    }

    fn call_method(
        &mut self,
        instance: &LoxInstance,
        method: Literal,
        args: Vec<Literal>,
        paren: &Token,
    ) -> Result<Literal> {
        if let Literal::FunctionValue(func) = method {
            // 创建新的闭包环境
            let mut closure = (*func.closure).clone();
            closure.define("this".into(), Literal::InstanceValue(instance.clone()));

            // 创建绑定实例后的函数
            let bound_func = LoxFunction {
                declaration: func.declaration,
                closure: Box::new(closure),
            };

            self.call_function(&bound_func, args, paren)
        } else {
            Err(RuntimeError::Runtime(
                paren.clone(),
                "Invalid method".into(),
            ))
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
            Err(RuntimeError::Runtime(
                op.clone(),
                "Operand must be a number".into(),
            ))
        }
    }

    // 实现加法（支持字符串连接）
    fn add_values(&self, a: &Literal, b: &Literal) -> Result<Literal> {
        match (a, b) {
            (Literal::NumberValue(n1), Literal::NumberValue(n2)) => {
                Ok(Literal::NumberValue(n1 + n2))
            }
            (Literal::StringValue(s1), Literal::StringValue(s2)) => {
                Ok(Literal::StringValue(format!("{}{}", s1, s2)))
            }
            _ => Err(RuntimeError::Runtime(
                Token::new(TokenType::Plus, 0, "+".into(), None),
                "Operands must be two numbers or two strings".into(),
            )),
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

            // 数值比较
            (Literal::NumberValue(a), Literal::NumberValue(b)) => (a - b).abs() < f64::EPSILON,

            // 字符串内容比较
            (Literal::StringValue(a), Literal::StringValue(b)) => a == b,

            // 函数比较（指针地址比较）
            (Literal::FunctionValue(a), Literal::FunctionValue(b)) => std::ptr::eq(a, b),

            // 类比较
            (Literal::ClassValue(a), Literal::ClassValue(b)) => {
                a.name == b.name && std::ptr::eq(a, b)
            }

            // 其他情况均为不相等
            _ => false,
        }
    }

    fn as_bool(&self, val: &Literal, op: &Token) -> Result<bool> {
        match val {
            Literal::Boolean(b) => Ok(*b),
            _ => Err(RuntimeError::Runtime(
                op.clone(),
                format!("Operand must be boolean (got {})", val.type_name()),
            )),
        }
    }

    // 逻辑与运算
    fn logical_and(&self, a: &Literal, b: &Literal, op: &Token) -> Result<Literal> {
        let a_bool = self.as_bool(a, op)?;
        let b_bool = self.as_bool(b, op)?;
        Ok(Literal::Boolean(a_bool && b_bool))
    }

    // 逻辑或运算
    fn logical_or(&self, a: &Literal, b: &Literal, op: &Token) -> Result<Literal> {
        let a_bool = self.as_bool(a, op)?;
        let b_bool = self.as_bool(b, op)?;
        Ok(Literal::Boolean(a_bool || b_bool))
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
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
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
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
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
            Stmt::Function {
                name,
                params: _,
                body: _,
            } => {
                // 将函数存储为环境中的可调用对象
                let function = LoxFunction {
                    declaration: Box::new(stmt.clone()), // 必须显式装箱
                    closure: Box::new(self.environment.deep_clone()),
                };
                self.environment
                    .define(name.lexeme.clone(), Literal::FunctionValue(function));
                Ok(())
            }

            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                // 解析超类（如果有）
                let super_class = match superclass {
                    Some(expr) => {
                        let val = self.evaluate(expr)?;
                        match val {
                            Literal::ClassValue(c) => Some(Box::new(c)),
                            _ => {
                                return Err(RuntimeError::Runtime(
                                    name.clone(),
                                    "Superclass must be a class".into(),
                                ));
                            }
                        }
                    }
                    None => None,
                };

                let class = LoxClass {
                    name: name.lexeme.clone(),
                    methods: methods.clone(),
                    superclass: super_class,
                    closure: Box::new(self.environment.deep_clone()),
                };

                self.environment
                    .define(name.lexeme.clone(), Literal::ClassValue(class));
                Ok(())
            }

            Stmt::Return { keyword: _, value } => {
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
            Some(Box::new(previous)), // 直接使用克隆的环境
        ));
        let result = stmts.iter().try_for_each(|stmt| self.execute(stmt));
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
            Literal::FunctionValue(f) => format!(
                "<fn {}>",
                match *f.declaration {
                    Stmt::Function { name, .. } => name.lexeme,
                    _ => "anonymous".into(),
                }
            ),
            Literal::ClassValue(c) => format!("<class {}>", c.name),
            Literal::InstanceValue(i) => format!("<instance of {}>", i.class.name),
            Literal::None => "nil".into(), // 合并None和Nil处理
        }
    }

    // 修改call_function函数签名和逻辑
    // interpreter.rs中修改call_function函数
    fn call_function(
        &mut self,
        func: &LoxFunction,
        args: Vec<Literal>,
        paren: &Token,
    ) -> Result<Literal> {
        // 提取函数参数和body
        let (params, body) = match func.declaration.as_ref() {
            Stmt::Function { params, body, .. } => (params, body),
            _ => {
                return Err(RuntimeError::Runtime(
                    paren.clone(),
                    "Invalid function declaration".into(),
                ));
            }
        };

        // 参数数量检查
        if args.len() != params.len() {
            return Err(RuntimeError::Runtime(
                paren.clone(),
                format!("Expected {} arguments but got {}", params.len(), args.len()),
            ));
        }

        // 创建闭包环境
        let mut env = Environment::new(Some(Box::new((*func.closure).clone())));

        // 绑定参数
        for (param, arg) in params.iter().zip(args) {
            env.define(param.lexeme.clone(), arg);
        }

        // 执行函数体
        let prev_env = self.environment.clone();
        self.environment = Box::new(env);

        let result = self.execute_block(body); // 这里可以正确获取body
        self.environment = prev_env;

        // 处理返回值
        match result {
            Ok(()) => Ok(Literal::Nil),
            Err(RuntimeError::Return(value)) => Ok(value),
            Err(e) => Err(e),
        }
    }

    fn call_class_constructor(
        &mut self,
        cls: &LoxClass,
        args: Vec<Literal>,
        paren: &Token,
    ) -> Result<Literal> {
        // 直接使用已解析的类定义
        let instance = Literal::InstanceValue(LoxInstance {
            class: cls.clone(),
            fields: HashMap::new(),
        });

        // 绑定 this 到闭包环境
        let mut init_closure = (*cls.closure).clone();
        init_closure.define("this".into(), instance.clone());

        // 自动调用 init 方法
        if let Some(Literal::FunctionValue(init_method)) = cls.find_method("init") {
            let bound_init = LoxFunction {
                declaration: init_method.declaration.clone(),
                closure: Box::new(init_closure),
            };
            self.call_function(&bound_init, args, paren)?;
        }

        Ok(instance)
    }
}
