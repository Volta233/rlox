use serde::Serialize;
use std::collections::HashMap;
use crate::statement::Stmt;
use crate::environment::Environment;


#[derive(Debug, Clone, Serialize)]
pub struct LoxFunction {
    pub declaration: Box<Stmt>, // 使用Box包装语句
    pub closure: Box<Environment>, // 使用Box包装环境
}

#[derive(Debug, Serialize)]
pub struct LoxClass {
    pub name: String,
    pub methods: Vec<Stmt>,
    pub superclass: Option<Box<LoxClass>>,
    #[serde(skip)]
    pub closure: Box<Environment>,
}

// 手动实现 Clone（不能使用 derive 因为 Environment 需要深度克隆）
impl Clone for LoxClass {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            methods: self.methods.clone(),
            superclass: self.superclass.clone(),
            closure: Box::new((*self.closure).clone()), 
        }
    }
}


impl LoxClass {
    pub fn find_method(&self, name: &str) -> Option<Literal> {
        // 在当前类中查找
        for method in &self.methods {
            if let Stmt::Function { name: method_name, .. } = method {
                if method_name.lexeme == name {
                    return Some(Literal::FunctionValue(LoxFunction {
                        declaration: Box::new(method.clone()),
                        closure: self.closure.clone(),
                    }));
                }
            }
        }

        // 查找超类链
        if let Some(ref superclass) = self.superclass {
            return superclass.find_method(name);
        }

        None
    }

    // 新增方法用于检查是否是某类的子类
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
        let mut closure = (*self.closure).clone();
        closure.define("this".into(), Literal::InstanceValue(instance.clone()));
        
        LoxFunction {
            declaration: self.declaration.clone(),
            closure: Box::new(closure),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoxInstance {
    pub class: LoxClass,
    pub fields: HashMap<String, Literal>,
}

impl LoxInstance {
    pub fn debug_print_fields(&self){
        if self.fields.is_empty() {
            println!("[DEBUG] Instance fields: (empty)");
        } else {
            for (k, v) in &self.fields {
                println!("[DEBUG] Field '{}': {:?}", k, v);
            }
        }
    }
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
}