use crate::environment::{Environment, RuntimeError};
use crate::expr::Expr;
use crate::statement::Stmt;
use crate::token::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    instance_counter: usize, // 新增实例计数器
}

impl Interpreter {

    fn get_call_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::GetAttribute { name, .. } => name.lexeme.clone(),
            _ => String::new(),
        }
    }

    fn evaluate_args(&mut self, exprs: &[Expr]) -> Result<Vec<Literal>> {
        // 移除显式错误类型
        exprs.iter().map(|expr| self.evaluate(expr)).collect()
    }

    pub fn new() -> Self {
        // 预定义全局函数（如clock）
        let env = Environment::new(None);
        // 定义 clock 函数（返回自 Unix 纪元以来的秒数）
        env.borrow_mut().define("clock".to_string(), Literal::NativeFunctionValue(|args| {
            // 参数检查
            if !args.is_empty() {
                return Err(RuntimeError::Runtime(
                    format!("Expected 0 arguments but got {}.", args.len()),
                ));
            }
            
            // 计算当前时间
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| RuntimeError::Runtime(
                    "SystemTime error.".to_string(),
                ))?;
            Ok(Literal::NumberValue(now.as_secs_f64()))
        }));

        Self {
            environment: env,
            instance_counter: 0,
        }
    }

    // 主控流程，解释每一个表达式
    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        for stmt in statements {
            self.execute(stmt)?;

        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Variable { name } => self.environment.borrow().get(name),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::Minus => self
                        .check_number_operand(&right_val)
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
                    TokenType::Minus => self.sub_numbers(&left_val, &right_val),
                    TokenType::Star => self.mul_numbers(&left_val, &right_val),
                    TokenType::Slash => self.div_numbers(&left_val, &right_val),
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
                    TokenType::And => self.logical_and(&left_val, &right_val),
                    TokenType::Or => self.logical_or(&left_val, &right_val),
                    _ => Err(RuntimeError::Runtime(
                        "Invalid operator.".into(),
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
                        Ok(instance)
                    }
                    // 处理实例方法调用
                    Literal::InstanceValue(inst) => {
                        let method_name = self.get_call_name(callee);
                        // println!("flag for this.");
                        if let Some(Literal::FunctionValue(func)) = inst.class.find_method(&method_name) {
                            let bound_func = func.bind(&inst);
                            self.call_function(&bound_func, args, paren)
                        } else {
                            Err(RuntimeError::Runtime(
                                format!("Undefined property '{}'.", method_name),
                            ))
                        }
                    }
                    Literal::NativeFunctionValue(func) => {
                        // 调用原生函数
                        func(&args)
                    }
                    _ => Err(RuntimeError::Runtime(
                        "Can only call functions and classes.".into(),
                    )),
                }
            }
            Expr::Super { keyword, method } => {
                // 步骤1：获取超类引用
                let super_class = match self.environment.borrow().get(keyword)? {
                    Literal::ClassValue(c) => c,
                    _ => {
                        return Err(RuntimeError::Runtime(
                            "Invalid super class.".into(),
                        ));
                    }
                };

                // 步骤2：获取当前实例的this绑定
                let this_instance = match self.environment.borrow().get(&Token::this())? {
                    Literal::InstanceValue(i) => i,
                    _ => {
                        return Err(RuntimeError::Runtime(
                            "super must be used in instance method.".into(),
                        ));
                    }
                };

                // 步骤3：查找超类方法
                let method = super_class.find_method(&method.lexeme).ok_or_else(|| {
                    RuntimeError::Runtime(
                        format!("Undefined property '{}'.", method.lexeme),
                    )
                })?;

                // 步骤4：创建闭包环境
                if let Literal::FunctionValue(func) = method {
                    let bound_func = func.bind(&this_instance);
                    Ok(Literal::FunctionValue(bound_func))
                } else {
                    // 使用调用方法时的方法名 Token 来构建错误
                    Err(RuntimeError::Runtime(
                        format!("'{}' is not a function.", keyword.lexeme).into(),
                    ))
                }
            }
            Expr::GetAttribute { object, name } => {
                let obj = self.evaluate(object)?;
                if let Literal::InstanceValue(instance) = obj {
                    // 尝试获取字段
                    match instance.environment.borrow_mut().get(name) {
                        Ok(field) => Ok(field),
                        Err(_) => {
                            // 字段不存在，查找方法并绑定实例
                            // println!("flag2 for this.");
                            if let Some(Literal::FunctionValue(func)) = instance.class.find_method(&name.lexeme) {
                                let bound_func = func.bind(&instance);
                                Ok(Literal::FunctionValue(bound_func))
                            } else {
                                Err(RuntimeError::Runtime(
                                    format!("Undefined property '{}'.", name.lexeme),
                                ))
                            }
                        }
                    }
                } else {
                    Err(RuntimeError::Runtime(
                        "Only instances have attributes.".into(),
                    ))
                }
            }
            // 变量赋值表达式
            Expr::Assign { name, value } => {
                let val = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, val.clone())?;
                Ok(val)
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let obj = self.evaluate(object)?;
                let val = self.evaluate(value)?;

                if let Literal::InstanceValue(instance) = obj {
                    instance.environment.borrow_mut().define(name.lexeme.clone(), val.clone());
                    Ok(Literal::InstanceValue(instance))
                } else {
                    Err(RuntimeError::Runtime(
                        "Only instances can have fields.".into(),
                    ))
                }
            }
            Expr::This { keyword } => {
                // 从当前环境获取this绑定
                // self.environment.check_this_binding(format!("Checking 'this' at line {}", keyword.line));
                let this_value = self.environment.borrow().get(keyword)?;

                // 验证必须是实例类型
                if let Literal::InstanceValue(instance) = this_value {
                    Ok(Literal::InstanceValue(instance))
                } else {
                    Err(RuntimeError::Runtime(
                        "Invalid 'this' context.".into(),
                    ))
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

    fn check_number_operand(&self, val: &Literal) -> Result<f64> {
        if let Literal::NumberValue(n) = val {
            Ok(*n)
        } else {
            Err(RuntimeError::Runtime(
                "Operand must be a number.".into(),
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
                "Operands must be two numbers or two strings.".into(),
            )),
        }
    }

    fn sub_numbers(&self, left: &Literal, right: &Literal) -> Result<Literal> {
        let (a, b) = self.check_number_operands(left, right)?;
        Ok(Literal::NumberValue(a - b))
    }

    fn mul_numbers(&self, left: &Literal, right: &Literal) -> Result<Literal> {
        let (a, b) = self.check_number_operands(left, right)?;
        Ok(Literal::NumberValue(a * b))
    }

    fn div_numbers(&self, left: &Literal, right: &Literal) -> Result<Literal> {
        let (a, b) = self.check_number_operands(left, right)?;
        if b == 0.0 {
            return Err(RuntimeError::Runtime("Division by zero.".into()));
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

    fn as_bool(&self, val: &Literal) -> Result<bool> {
        match val {
            Literal::Boolean(b) => Ok(*b),
            _ => Err(RuntimeError::Runtime(
                format!("Operand must be boolean (got {}).", val.type_name()),
            )),
        }
    }

    // 逻辑与运算
    fn logical_and(&self, a: &Literal, b: &Literal) -> Result<Literal> {
        let a_bool = self.as_bool(a)?;
        let b_bool = self.as_bool(b)?;
        Ok(Literal::Boolean(a_bool && b_bool))
    }

    // 逻辑或运算
    fn logical_or(&self, a: &Literal, b: &Literal) -> Result<Literal> {
        let a_bool = self.as_bool(a)?;
        let b_bool = self.as_bool(b)?;
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
                "Operands must be numbers or strings.".into(),
            )),
        }
    }

    // 公共类型检查方法
    fn check_number_operands(
        &self,
        left: &Literal,
        right: &Literal,
    ) -> Result<(f64, f64)> {
        if let (Literal::NumberValue(a), Literal::NumberValue(b)) = (left, right) {
            Ok((*a, *b))
        } else {
            Err(RuntimeError::Runtime(
                "Operands must be two numbers or two strings.".into(),
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
                if self.environment.borrow().values.contains_key(&name.lexeme) {
                    return Err(RuntimeError::Runtime(
                        format!("Error: Already a variable with name '{}' in this scope.", name.lexeme)
                    ));
                }
                
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Literal::Nil,
                };
                self.environment.borrow_mut().define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block { statements } => {
                // 保存当前环境
                let previous_env = self.environment.clone();
                
                // 创建新环境（继承当前环境）
                self.environment = Environment::new(Some(previous_env));
                
                // 执行块内语句
                let result = self.execute_block(statements);
                
                // 恢复父环境
                let restore = self.environment.borrow().enclosing.as_ref().unwrap().clone();
                self.environment = restore;
                
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_result = self.evaluate(condition)?;
                if self.is_truthy(&cond_result) {
                    self.execute(then_branch)
                } else {
                    else_branch.as_ref().map_or(Ok(()), |e| self.execute(e))
                }
            }
            Stmt::While { condition, body } => {
                while {
                    let cond = self.evaluate(condition)?;
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
                params,
                body,
            } => {
                // 创建闭包环境（继承当前环境）
                let closure_env = Environment::new(Some(self.environment.clone()));
                
                // 预定义函数名（用于递归）
                closure_env.borrow_mut().define(name.lexeme.clone(), Literal::Nil);

                // 创建函数对象
                let func = LoxFunction {
                    params: params.clone(),
                    body: body.clone(),
                    closure: closure_env.clone(),
                    is_initializer: false,
                };

                // 更新闭包环境中的函数引用
                closure_env.borrow_mut().assign(
                    &Token::new_identifier(name.lexeme.clone()),
                    Literal::FunctionValue(func.clone())
                )?;

                // 将函数绑定到当前环境
                self.environment.borrow_mut().define(name.lexeme.clone(), Literal::FunctionValue(func));
                Ok(())
            }

            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                // 解析超类
                let super_class = match superclass {
                    Some(expr) => {
                        let val = self.evaluate(expr)?;
                        match val {
                            Literal::ClassValue(c) => Some(Box::new(c)),
                            _ => {
                                return Err(RuntimeError::Runtime(
                                    "Superclass must be a class.".into(),
                                ));
                            }
                        }
                    }
                    None => None,
                };

                // 创建类环境（继承当前环境）
                let class_env = Environment::new(Some(self.environment.clone()));
                
                // 如果有超类，将super绑定到超类
                if let Some(super_class) = &super_class {
                    class_env.borrow_mut().define(
                        "super".to_string(),
                        Literal::ClassValue((**super_class).clone()),
                    );
                }
                
                // 将方法存入类环境
                for method in methods {
                    if let Stmt::Function {
                        name: method_name,
                        params,
                        body,
                    } = method {
                        let func = LoxFunction {
                            params: params.clone(),
                            body: body.clone(),
                            closure: class_env.clone(), // 直接使用 Rc 克隆
                            is_initializer: method_name.lexeme == "init",
                        };
                        class_env.borrow_mut().define(
                            method_name.lexeme.clone(),
                            Literal::FunctionValue(func),
                        );
                    }
                }

                // 创建类对象
                let class = LoxClass {
                    name: name.lexeme.clone(),
                    environment: class_env,
                    superclass: super_class,
                };

                self.environment.borrow_mut().define(name.lexeme.clone(), Literal::ClassValue(class));
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
        let previous = Rc::clone(&self.environment);
        self.environment = Environment::new(Some(previous));
        let result = stmts.iter().try_for_each(|stmt| self.execute(stmt));
        let cur_environment = self.environment.borrow().enclosing.as_ref().unwrap().clone();
        self.environment = cur_environment;
        result
    }


    // 调用函数时使用
    fn call_function(
        &mut self,
        func: &LoxFunction,
        args: Vec<Literal>,
        _paren: &Token,
    ) -> Result<Literal> {
        let call_env = Environment::new(Some(Rc::clone(&func.closure)));
    
        // 绑定参数
        for (param, arg) in func.params.iter().zip(args.iter()) {
            call_env.borrow_mut().define(param.lexeme.clone(), arg.clone());
        }

        // 执行函数体
        let prev_env = Rc::clone(&self.environment);
        self.environment = Rc::clone(&call_env);
        let result = self.execute_block(&func.body);
        self.environment = prev_env;

        if func.is_initializer {
            Ok(Literal::Nil) // 返回值被call_class_constructor忽略
        } else {
            match result {
                Ok(_) => Ok(Literal::Nil),
                Err(RuntimeError::Return(value)) => Ok(value),
                Err(e) => Err(e),
            }
        }
    }

    // 新建一个实例时调用
    fn call_class_constructor(
        &mut self,
        cls: &LoxClass,
        args: Vec<Literal>,
        paren: &Token,
    ) -> Result<Literal> {
        let instance_name = format!("{}#{}", cls.name, self.instance_counter);
        self.instance_counter += 1;

        let instance_env = Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        }));
        
        // 从类环境中复制super绑定到实例环境
        if let Ok(super_value) = cls.environment.borrow().get(
            &Token::new_identifier("super".to_string())
        ) {
            instance_env.borrow_mut().define(
                "super".to_string(),
                super_value
            );
        }

        let instance = LoxInstance {
            class: cls.clone(),
            environment: instance_env,
            name: instance_name,
        };

        // 自动调用初始化方法
        if let Some(Literal::FunctionValue(init)) = cls.find_method("init") {
            let bound_init = init.bind(&instance);
            self.call_function(&bound_init, args, paren)?;
        }
        Ok(Literal::InstanceValue(instance))
    }

    pub fn debug_print_env(&self) {
        self.environment.borrow().debug_print(0);
    }

    fn stringify(&self, value: Literal) -> String {
        match value {
            Literal::Nil => "nil".into(),
            Literal::Boolean(b) => b.to_string(),
            Literal::NumberValue(n) => format!("{}", n),
            Literal::StringValue(s) => s,
            Literal::FunctionValue(_) => "call fn".into(),
            Literal::ClassValue(c) => format!("<class {}>", c.name),
            Literal::InstanceValue(i) => format!("<instance of {}>", i.class.name),
            Literal::None => "nil".into(), // 合并None和Nil处理
            Literal::NativeFunctionValue(_) => "call native fn".into(),
        }
    }
}
