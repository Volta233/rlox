use serde::Serialize;
use crate::statement::Stmt;
use crate::environment::Environment;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Serialize)]
pub struct LoxFunction {
    pub params: Vec<Token>,      // 参数列表
    pub body: Vec<Stmt>,         // 函数体
    pub closure: Box<Environment>, // 闭包环境
    pub is_initializer: bool,    // 是否是初始化方法
}

#[derive(Debug, Serialize)]
pub struct LoxClass {
    pub name: String,
    pub environment: Environment, // 方法存储在环境中
    pub superclass: Option<Box<LoxClass>>,
}

impl Clone for LoxClass {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            environment: self.environment.deep_clone(),
            superclass: self.superclass.clone(),
        }
    }
}

impl LoxClass {
    pub fn find_method(&self, name: &str) -> Option<Literal> {
        if let Ok(Literal::FunctionValue(func)) = self.environment.get(&Token::new_identifier(name.to_string())) {
            func.closure.check_this_binding("Found method in class".to_string());
        }
        // 在当前类环境查找
        match self.environment.get(&Token::new_identifier(name.to_string())) {
            Ok(Literal::FunctionValue(func)) => Some(Literal::FunctionValue(func)),
            Ok(_) => None,
            Err(_) => {
                // 递归查找超类链
                self.superclass.as_ref().and_then(|s| s.find_method(name))
            }
        }
    }

    pub fn is_subclass_of(&self, other: &LoxClass) -> bool {
        if let Some(ref superclass) = self.superclass {
            if superclass.name == other.name {
                return true;
            }
            return superclass.is_subclass_of(other);
        }
        false
    }
}

// 为方法调用添加辅助方法
impl LoxFunction {
    pub fn bind(&self, instance: &LoxInstance) -> Self {
        let mut closure = (*self.closure).deep_clone();
        closure.define("this".into(), Literal::InstanceValue(instance.clone()));
        // 检查闭包环境是否包含this
        closure.check_this_binding("After binding in LoxFunction::bind".to_string());
        
        LoxFunction {
            params: self.params.clone(),
            body: self.body.clone(),
            closure: Box::new(closure),
            is_initializer: self.is_initializer,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoxInstance {
    pub class: LoxClass,
    pub environment: Rc<RefCell<Environment>>, // 使用Rc和RefCell共享环境
    pub name: String, // 新增 name 字段
}

#[derive(Debug, Clone, Serialize)]
pub enum Literal {
    StringValue(String),
    NumberValue(f64),
    Boolean(bool),
    Nil,
    FunctionValue(LoxFunction),
    ClassValue(LoxClass),
    InstanceValue(LoxInstance),
    None,
}

impl Literal {
    pub fn as_instance(&self) -> Option<&LoxInstance> {
        if let Literal::InstanceValue(i) = self {
            Some(i)
        } else {
            None
        }
    }
    pub fn as_class(&self) -> Option<LoxClass> {
        if let Literal::ClassValue(c) = self {
            Some(c.clone())
        } else {
            None
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Literal::StringValue(_) => "string",
            Literal::NumberValue(_) => "number",
            Literal::Boolean(_) => "boolean",
            Literal::Nil => "nil",
            Literal::FunctionValue(_) => "function",
            Literal::ClassValue(_) => "class",
            Literal::InstanceValue(_) => "instance",
            Literal::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenType {
    // --- 单字符符号 ---
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // --- 一或两个字符符号 ---
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // --- 字面量 ---
    Identifier,
    String,
    Number,

    // --- 关键字 ---
    And,    
    Class,
    Else,
    False,  
    Fun,
    For,
    If,
    Nil,    
    Or,    
    Print,  
    Return,
    Super,  
    This,   
    True,   
    Var,
    While,

    // --- 错误类型 ---
    Error,

    // --- 其他 ---
    Eof,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub lexeme: String,
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(
        token_type: TokenType, 
        line: usize, 
        lexeme: String,
        literal: Option<Literal>
    ) -> Self {
        Self {
            token_type,
            line,
            lexeme,
            literal,
        }
    }

    pub fn this() -> Self {
        Self {
            token_type: TokenType::This,
            line: 0,
            lexeme: "this".into(),
            literal: None,
        }
    }

    pub fn new_identifier(name: String) -> Self {
        Self {
            token_type: TokenType::Identifier,
            line: 0, // 实际使用时应传入正确的行号
            lexeme: name.clone(),
            literal: Some(Literal::StringValue(name)),
        }
    }
}