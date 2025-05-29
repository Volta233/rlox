use std::collections::HashMap;
use serde::Serialize;
use crate::token::{Token, Literal};
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::error::Error;

#[derive(Debug)]
pub enum RuntimeError {
    Return(Literal),  // 处理return语句
    Runtime(String),  // (错误token, 错误信息)
}

// 实现 Display 提供错误描述
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::Return(_) => write!(f, "Return statement correctly."),
            RuntimeError::Runtime(msg) => 
                write!(f, "RuntimeError: {}", msg),
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
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing,
        }))
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal> {
        let key = &name.lexeme;
        if let Some(val) = self.values.get(key) {
            Ok(val.clone())
        } else if let Some(env) = &self.enclosing {
            env.borrow().get(name)
        } else {
            // 特殊处理this关键字
            if key == "this" {
                Err(RuntimeError::Runtime("this isn't bound in environment.".into()))
            } else {
                Err(RuntimeError::Runtime(format!("Undefined variable '{}'.", key)))
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<()> {
        let key = &name.lexeme;
        if self.values.contains_key(key) {
            self.values.insert(key.to_string(), value);
            Ok(())
        } else if let Some(env) = &mut self.enclosing {
            env.borrow_mut().assign(name, value)
        } else {
            Err(RuntimeError::Runtime(format!("Undefined variable '{}'.", key)))
        }
    }

    /// 检查当前环境链中是否存在 "this" 绑定
    pub fn has_this(&self) -> bool {
        // 检查当前环境
        if self.values.contains_key("this") {
            return true;
        }

        // 递归检查父环境
        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().has_this()
        } else {
            false
        }
    }

    /// 调试函数：检查当前环境链是否有 "this" 绑定
    pub fn check_this_binding(&self, msg: String) {
        let has_this = self.has_this();
        if !has_this {
            println!("[DEBUG] ❌ No 'this' binding: {}", msg);
        } else {
            match self.get(&Token::this()) {
                Ok(Literal::InstanceValue(inst)) => {
                    println!("[DEBUG] ✅ Has 'this' binding: {} | Instance: {}", msg, inst.name);
                }
                _ => println!("[DEBUG] ⚠️ Invalid 'this' binding: {}", msg),
            }
        }
    }

    pub fn debug_print(&self, depth: usize) {
        println!("🛠️  Environment Depth {}:", depth);
        for (key, val) in &self.values {
            match val {
                Literal::InstanceValue(inst) => {
                    println!("   🔑 {} => 🏷️ {} (Instance of {})", 
                        key, inst.name, inst.class.name);
                }
                Literal::ClassValue(cls) => {
                    println!("   🔑 {} => 🏛️ {}", key, cls.name);
                }
                _ => println!("   🔑 {} => {:?}", key, val),
            }
        }
        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().debug_print(depth + 1);
        }
    }

    pub fn debug_loc(&self) -> String {
        if self.values.contains_key("this") {
            "[有 this 绑定]".into()
        } else if let Some(env) = &self.enclosing {
            env.borrow().debug_loc()
        } else {
            "[全局环境]".into()
        }
    }
}