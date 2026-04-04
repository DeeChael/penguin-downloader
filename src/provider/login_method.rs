use std::time::Duration;
use crate::provider::{LoginMethodType, QrLoginCallback, UrlLoginCallback, CodeLoginCallback};
use crate::Result;

pub trait LoginMethod: Send + Sync {
    fn id(&self) -> &str;
    fn method_type(&self) -> LoginMethodType;
    fn name(&self) -> &str;
}

pub struct QrLoginMethod {
    id: String,
    name: String,
    qr_handler: Box<dyn Fn(&dyn QrLoginCallback, Duration) -> Result<String> + Send + Sync>,
}

impl QrLoginMethod {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        qr_handler: Box<dyn Fn(&dyn QrLoginCallback, Duration) -> Result<String> + Send + Sync>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            qr_handler,
        }
    }

    pub async fn start_login(&self, callback: &dyn QrLoginCallback, timeout: Duration) -> Result<String> {
        (self.qr_handler)(callback, timeout)
    }
}

impl LoginMethod for QrLoginMethod {
    fn id(&self) -> &str {
        &self.id
    }

    fn method_type(&self) -> LoginMethodType {
        LoginMethodType::QR
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct UrlLoginMethod {
    id: String,
    name: String,
    url_handler: Box<dyn Fn(&dyn UrlLoginCallback, Duration) -> Result<String> + Send + Sync>,
}

impl UrlLoginMethod {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        url_handler: Box<dyn Fn(&dyn UrlLoginCallback, Duration) -> Result<String> + Send + Sync>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url_handler,
        }
    }

    pub async fn start_login(&self, callback: &dyn UrlLoginCallback, timeout: Duration) -> Result<String> {
        (self.url_handler)(callback, timeout)
    }
}

impl LoginMethod for UrlLoginMethod {
    fn id(&self) -> &str {
        &self.id
    }

    fn method_type(&self) -> LoginMethodType {
        LoginMethodType::URL
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct AccountLoginMethod {
    id: String,
    name: String,
    account_handler: Box<dyn Fn(&str, &str, Duration) -> Result<String> + Send + Sync>,
}

impl AccountLoginMethod {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        account_handler: Box<dyn Fn(&str, &str, Duration) -> Result<String> + Send + Sync>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            account_handler,
        }
    }

    pub async fn start_login(&self, username: &str, password: &str, timeout: Duration) -> Result<String> {
        (self.account_handler)(username, password, timeout)
    }
}

impl LoginMethod for AccountLoginMethod {
    fn id(&self) -> &str {
        &self.id
    }

    fn method_type(&self) -> LoginMethodType {
        LoginMethodType::Account
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct CodeLoginMethod {
    id: String,
    name: String,
    code_handler: Box<dyn Fn(&str, &dyn CodeLoginCallback, Duration) -> Result<String> + Send + Sync>,
}

impl CodeLoginMethod {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        code_handler: Box<dyn Fn(&str, &dyn CodeLoginCallback, Duration) -> Result<String> + Send + Sync>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            code_handler,
        }
    }

    pub async fn start_login(&self, account: &str, callback: &dyn CodeLoginCallback, timeout: Duration) -> Result<String> {
        (self.code_handler)(account, callback, timeout)
    }
}

impl LoginMethod for CodeLoginMethod {
    fn id(&self) -> &str {
        &self.id
    }

    fn method_type(&self) -> LoginMethodType {
        LoginMethodType::Code
    }

    fn name(&self) -> &str {
        &self.name
    }
}
