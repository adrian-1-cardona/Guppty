// === value.rs ===
// Values are the actual "things" that exist when your program is running.

#[derive(Debug, Clone)]
pub enum Value {
    GuppyString(String),
    GuppyChar(char),
    GuppyNumber(i64),
    GuppyFloat(f64),
    GuppyBool(bool),
    GuppyArray(Vec<Value>),
    Nothing,
}

impl Value {
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
            Value::Nothing => String::new(),
        }
    }

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

    #[allow(dead_code)]
    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::GuppyBool(b) => Ok(*b),
            other => Err(format!(
                "Expected a boolean but got {}",
                other.to_display_string()
            )),
        }
    }
}
