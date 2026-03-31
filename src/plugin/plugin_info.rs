#[derive(Debug, Clone)]
pub enum ProviderOptionType {
    String,
    Integer,
    Boolean,
    Float,
}

#[derive(Debug, Clone)]
pub struct ProviderOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub option_type: ProviderOptionType,
    pub required: bool,
    pub default_value: Option<String>,
}

impl ProviderOption {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        option_type: ProviderOptionType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            option_type,
            required: false,
            default_value: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub options: Vec<ProviderOption>,
}

impl PluginInfo {
    pub fn validate_id(id: &str) -> bool {
        if id.is_empty() {
            return false;
        }

        if let Some(first) = id.chars().next() {
            if first.is_ascii_digit() {
                return false;
            }
        }

        id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let id = id.into();
        assert!(
            Self::validate_id(&id),
            "Plugin ID must start with a letter or underscore and contain only letters, numbers, and underscores"
        );

        Self {
            id,
            name: name.into(),
            version: version.into(),
            description: description.into(),
            options: Vec::new(),
        }
    }

    pub fn with_options(mut self, options: Vec<ProviderOption>) -> Self {
        self.options = options;
        self
    }

    pub fn add_option(mut self, option: ProviderOption) -> Self {
        self.options.push(option);
        self
    }
}
