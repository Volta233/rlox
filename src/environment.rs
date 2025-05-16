use std::collections::HashMap;
use serde::Serialize;
use crate::token::{Token, Literal};
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum RuntimeError {
    Return(Literal),  // 处理return语句
    Runtime(Token, String),  // (错误token, 错误信息)
}

// 实现 Display 提供错误描述
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::Return(_) => write!(f, "Return statement correctly."),
            RuntimeError::Runtime(token, msg) => 
                write!(f, "[Line {}] Runtime Error: {}", token.line, msg),
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None // 如果不需要错误链可留空
    }
}

type Result<T> = std::result::Result<T, RuntimeError>;
// 环境嵌套结构（支持作用域链）
#[derive(Debug, Clone, Serialize)]
pub struct Environment {
    pub values: HashMap<String, Literal>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal> {
        let key = &name.lexeme;
        if let Some(val) = self.values.get(key) {
            Ok(val.clone())
        } else if let Some(env) = &self.enclosing {
            env.get(name)
        } else {
            // 特殊处理this关键字
            if key == "this" {
                Err(RuntimeError::Runtime(name.clone(), "Invalid 'this' context".into()))
            } else {
                Err(RuntimeError::Runtime(name.clone(), format!("Undefined variable '{}'", key)))
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<()> {
        let key = &name.lexeme;
        if self.values.contains_key(key) {
            self.values.insert(key.to_string(), value);
            Ok(())
        } else if let Some(env) = &mut self.enclosing {
            env.assign(name, value)
        } else {
            Err(RuntimeError::Runtime(name.clone(), format!("Undefined variable '{}'", key)))
        }
    }

    pub fn deep_clone(&self) -> Environment {
        Environment {
            values: self.values.clone(),
            enclosing: self.enclosing.as_ref().map(|e| Box::new(e.deep_clone())),
        }
    }
}