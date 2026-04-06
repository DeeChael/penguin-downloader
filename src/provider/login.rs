use crate::provider::types::LoginMethodType;
use crate::Result;
use std::time::Duration;

// ============== Data Types ==============

#[derive(Debug, Clone)]
pub enum LoginStatus {
    Pending,
    Scanned,
    Success,
    Failed(String),
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone)]
pub enum QrLoginData {
    Image(Vec<u8>),
    Url(String),
}

// ============== Callbacks ==============

pub trait QrLoginCallback: Send + Sync {
    fn on_qr_data(&self, data: QrLoginData);
    fn clone_box(&self) -> Box<dyn QrLoginCallback>;
}

impl Clone for Box<dyn QrLoginCallback> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait UrlLoginCallback: Send + Sync {
    fn on_url(&self, url: String);
    fn clone_box(&self) -> Box<dyn UrlLoginCallback>;
}

impl Clone for Box<dyn UrlLoginCallback> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait CodeLoginCallback: Send + Sync {
    /// 请求验证码
    ///
    /// # Arguments
    /// * `url` - 可选的 URL，某些平台可能需要用户访问此 URL 完成人机验证
    fn request_code(&self, url: Option<&str>) -> String;
    fn clone_box(&self) -> Box<dyn CodeLoginCallback>;
}

impl Clone for Box<dyn CodeLoginCallback> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ============== Handlers ==============

/// Handler trait for QR login - synchronous interface
/// Implementations should internally handle async execution
pub trait QrLoginHandler: Send + Sync {
    fn handle(&self, callback: Box<dyn QrLoginCallback>, timeout: Duration) -> Result<String>;
}

/// Handler trait for URL login - synchronous interface
pub trait UrlLoginHandler: Send + Sync {
    fn handle(&self, callback: Box<dyn UrlLoginCallback>, timeout: Duration) -> Result<String>;
}

/// Handler trait for Account login - synchronous interface
pub trait AccountLoginHandler: Send + Sync {
    fn handle(&self, username: String, password: String, timeout: Duration) -> Result<String>;
}

/// Handler trait for Code login - synchronous interface
pub trait CodeLoginHandler: Send + Sync {
    fn handle(
        &self,
        account: String,
        callback: Box<dyn CodeLoginCallback>,
        timeout: Duration,
    ) -> Result<String>;
}

// ============== Methods ==============

pub trait LoginMethod: Send + Sync {
    fn id(&self) -> &str;
    fn method_type(&self) -> LoginMethodType;
    fn name(&self) -> &str;
}

pub struct QrLoginMethod {
    id: String,
    name: String,
    handler: Box<dyn QrLoginHandler>,
}

impl QrLoginMethod {
    pub fn new<H>(id: impl Into<String>, name: impl Into<String>, handler: H) -> Self
    where
        H: QrLoginHandler + 'static,
    {
        Self {
            id: id.into(),
            name: name.into(),
            handler: Box::new(handler),
        }
    }

    pub fn start_login(
        &self,
        callback: Box<dyn QrLoginCallback>,
        timeout: Duration,
    ) -> Result<String> {
        self.handler.handle(callback, timeout)
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
    handler: Box<dyn UrlLoginHandler>,
}

impl UrlLoginMethod {
    pub fn new<H>(id: impl Into<String>, name: impl Into<String>, handler: H) -> Self
    where
        H: UrlLoginHandler + 'static,
    {
        Self {
            id: id.into(),
            name: name.into(),
            handler: Box::new(handler),
        }
    }

    pub fn start_login(
        &self,
        callback: Box<dyn UrlLoginCallback>,
        timeout: Duration,
    ) -> Result<String> {
        self.handler.handle(callback, timeout)
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
    handler: Box<dyn AccountLoginHandler>,
}

impl AccountLoginMethod {
    pub fn new<H>(id: impl Into<String>, name: impl Into<String>, handler: H) -> Self
    where
        H: AccountLoginHandler + 'static,
    {
        Self {
            id: id.into(),
            name: name.into(),
            handler: Box::new(handler),
        }
    }

    pub fn start_login(
        &self,
        username: String,
        password: String,
        timeout: Duration,
    ) -> Result<String> {
        self.handler.handle(username, password, timeout)
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
    handler: Box<dyn CodeLoginHandler>,
}

impl CodeLoginMethod {
    pub fn new<H>(id: impl Into<String>, name: impl Into<String>, handler: H) -> Self
    where
        H: CodeLoginHandler + 'static,
    {
        Self {
            id: id.into(),
            name: name.into(),
            handler: Box::new(handler),
        }
    }

    pub fn start_login(
        &self,
        account: String,
        callback: Box<dyn CodeLoginCallback>,
        timeout: Duration,
    ) -> Result<String> {
        self.handler.handle(account, callback, timeout)
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
