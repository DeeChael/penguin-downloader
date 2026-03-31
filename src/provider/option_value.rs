
#[derive(Debug, Clone)]
pub enum ProviderOptionValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Float(f64),
}

impl ProviderOptionValue {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            ProviderOptionValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ProviderOptionValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ProviderOptionValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ProviderOptionValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            ProviderOptionValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_integer(self) -> Option<i64> {
        match self {
            ProviderOptionValue::Integer(i) => Some(i),
            _ => None,
        }
    }

    pub fn into_boolean(self) -> Option<bool> {
        match self {
            ProviderOptionValue::Boolean(b) => Some(b),
            _ => None,
        }
    }

    pub fn into_float(self) -> Option<f64> {
        match self {
            ProviderOptionValue::Float(f) => Some(f),
            _ => None,
        }
    }
}

impl From<String> for ProviderOptionValue {
    fn from(s: String) -> Self {
        ProviderOptionValue::String(s)
    }
}

impl From<&str> for ProviderOptionValue {
    fn from(s: &str) -> Self {
        ProviderOptionValue::String(s.to_string())
    }
}

impl From<i64> for ProviderOptionValue {
    fn from(i: i64) -> Self {
        ProviderOptionValue::Integer(i)
    }
}

impl From<i32> for ProviderOptionValue {
    fn from(i: i32) -> Self {
        ProviderOptionValue::Integer(i as i64)
    }
}

impl From<bool> for ProviderOptionValue {
    fn from(b: bool) -> Self {
        ProviderOptionValue::Boolean(b)
    }
}

impl From<f64> for ProviderOptionValue {
    fn from(f: f64) -> Self {
        ProviderOptionValue::Float(f)
    }
}

impl From<f32> for ProviderOptionValue {
    fn from(f: f32) -> Self {
        ProviderOptionValue::Float(f as f64)
    }
}
