use std::collections::HashMap;

/// Provider option value - represents the actual value of an option
#[derive(Debug, Clone)]
pub enum ProviderOptionValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Float(f64),
    Enum(i64), // index of the selected enum variant
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

    pub fn as_enum(&self) -> Option<i64> {
        match self {
            ProviderOptionValue::Enum(i) => Some(*i),
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

    pub fn into_enum(self) -> Option<i64> {
        match self {
            ProviderOptionValue::Enum(i) => Some(i),
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

/// Range for numeric types
#[derive(Debug, Clone)]
pub struct NumberRange<T> {
    pub min: T,
    pub max: T,
}

impl Default for NumberRange<i64> {
    fn default() -> Self {
        Self {
            min: i64::MIN,
            max: i64::MAX,
        }
    }
}

impl Default for NumberRange<f64> {
    fn default() -> Self {
        Self {
            min: f64::MIN,
            max: f64::MAX,
        }
    }
}

/// Option type definition with constraints
#[derive(Debug, Clone)]
pub enum ProviderOptionType {
    String,
    Integer { range: NumberRange<i64> },
    Boolean,
    Float { range: NumberRange<f64> },
    Enum { variant_count: i64 },
}

impl ProviderOptionType {
    pub fn integer_with_range(min: i64, max: i64) -> Self {
        ProviderOptionType::Integer {
            range: NumberRange { min, max },
        }
    }

    pub fn float_with_range(min: f64, max: f64) -> Self {
        ProviderOptionType::Float {
            range: NumberRange { min, max },
        }
    }

    pub fn enum_with_count(count: i64) -> Self {
        ProviderOptionType::Enum {
            variant_count: count,
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, ProviderOptionType::Enum { .. })
    }
}

/// Provider option definition - describes an option that can be configured
#[derive(Debug, Clone)]
pub struct ProviderOptionDefinition {
    /// Option name: only lowercase letters, numbers, hyphens allowed.
    /// Must start with a letter. Hyphens cannot be at start or end.
    pub name: String,
    /// Data type with constraints
    pub option_type: ProviderOptionType,
    /// Description of the option
    pub description: String,
}

impl ProviderOptionDefinition {
    /// Creates a new option definition with name validation
    ///
    /// # Panics
    /// Panics if the name doesn't follow the naming rules:
    /// - Only lowercase letters (a-z), numbers (0-9), and hyphens (-) allowed
    /// - Must start with a letter
    /// - Cannot start or end with a hyphen
    pub fn new(
        name: impl Into<String>,
        option_type: ProviderOptionType,
        description: impl Into<String>,
    ) -> Self {
        let name = name.into();
        assert!(
            Self::validate_name(&name),
            "Option name '{}' is invalid. Name must:\n\
             - Start with a lowercase letter (a-z)\n\
             - Only contain lowercase letters (a-z), numbers (0-9), and hyphens (-)\n\
             - Not start or end with a hyphen",
            name
        );

        Self {
            name,
            option_type,
            description: description.into(),
        }
    }

    /// Validates the option name according to the rules:
    /// - Only lowercase letters (a-z), numbers (0-9), and hyphens (-)
    /// - Must start with a letter (a-z)
    /// - Cannot start or end with a hyphen
    fn validate_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Check first character is a lowercase letter
        if let Some(first) = name.chars().next() {
            if !first.is_ascii_lowercase() {
                return false;
            }
        }

        // Check last character is not a hyphen
        if name.ends_with('-') {
            return false;
        }

        // Check all characters are valid
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    /// Checks if this option is an enum type
    pub fn is_enum(&self) -> bool {
        self.option_type.is_enum()
    }
}

/// Enum variant mapping: index -> variant name
pub type EnumVariantMap = HashMap<i64, String>;
