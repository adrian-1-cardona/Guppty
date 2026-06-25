// === value.rs ===
// values are the things that actually exist when your program runs!
// numbers, words, true/false, and even functions are all values.

use crate::environment::GuppyFunction;

#[derive(Debug, Clone)]
pub enum Value {
    GuppyString(String),
    GuppyChar(char),
    GuppyNumber(i64),
    GuppyFloat(f64),
    GuppyBool(bool),
    GuppyArray(Vec<Value>),
    GuppyFunction(GuppyFunction),
    Nothing,
}

impl Value {
    /// how we print a value to the screen with out()
    pub fn to_display_string(&self) -> String {
        match self {
            Value::GuppyString(text) => text.clone(),
            Value::GuppyChar(ch) => ch.to_string(),
            Value::GuppyNumber(n) => n.to_string(),
            Value::GuppyFloat(f) => {
                let rounded = (f * 100.0).round() / 100.0;
                if rounded.fract() == 0.0 {
                    format!("{:.1}", rounded)
                } else {
                    rounded.to_string()
                }
            }
            Value::GuppyBool(b) => b.to_string(),
            Value::GuppyArray(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::GuppyFunction(_) => "<function>".to_string(),
            Value::Nothing => String::new(),
        }
    }

    /// turn a value into a number for math — only numbers count!
    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Value::GuppyNumber(n) => Ok(*n as f64),
            Value::GuppyFloat(f) => Ok(*f),
            other => Err(format!(
                "Expected a number but got {}",
                other.to_display_string()
            )),
        }
    }

    /// is this value "truthy"? false and nothing are the only lies!
    /// this matters SO much because if/while use it to pick a path.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::GuppyBool(false) => false,
            Value::Nothing => false,
            _ => true,
        }
    }
}
