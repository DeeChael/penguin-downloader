use std::sync::Arc;
use std::time::Duration;

use crate::error::CoreError;

pub type LoginResult = Result<String, CoreError>;

pub enum LoginMethod {
    Qr {
        id: String,
        handler: Arc<dyn QrLoginMethodHandler>,
    },
    Url {
        id: String,
        handler: Arc<dyn UrlLoginMethodHandler>,
    },
    Token {
        id: String,
        handler: Arc<dyn TokenLoginMethodHandler>,
    },
    UsernameAndPassword {
        id: String,
        handler: Arc<dyn UsernameAndPasswordLoginMethodHandler>,
    },
    UsernameAndCode {
        id: String,
        resend_after: u64,
        handler: Arc<dyn UsernameAndCodeLoginMethodHandler>,
    },
}

pub trait QrLoginMethodHandler: Send + Sync {
    fn login(&self, callback: Box<dyn QrLoginMethodCallback>, timeout: Duration) -> LoginResult;
}

pub trait QrLoginMethodCallback: Send + Sync {
    fn on_qr_data(&self, data: QrLoginData);
    fn clone_box(&self) -> Box<dyn QrLoginMethodCallback>;
}

pub enum QrLoginData {
    Image(Vec<u8>),
    Url(String),
}

pub trait UrlLoginMethodHandler: Send + Sync {
    fn login(&self, callback: Box<dyn UrlLoginMethodCallback>, timeout: Duration) -> LoginResult;
}

pub trait UrlLoginMethodCallback: Send + Sync {
    fn on_url(&self, url: String);
    fn clone_box(&self) -> Box<dyn UrlLoginMethodCallback>;
}

pub trait TokenLoginMethodHandler: Send + Sync {
    fn login(&self, token: &str, timeout: Duration) -> LoginResult;
}

pub trait UsernameAndPasswordLoginMethodHandler: Send + Sync {
    fn login(
        &self,
        username: &str,
        password: &str,
        callback: Box<dyn UsernameAndPasswordLoginMethodCallback>,
        timeout: Duration,
    ) -> LoginResult;
}

pub trait UsernameAndPasswordLoginMethodCallback: Send + Sync {
    fn on_captcha(&self, url: Option<String>) -> Option<serde_json::Value>;
    fn clone_box(&self) -> Box<dyn UsernameAndPasswordLoginMethodCallback>;
}

pub trait UsernameAndCodeLoginMethodHandler: Send + Sync {
    fn login(
        &self,
        username: &str,
        callback: Box<dyn UsernameAndCodeLoginMethodCallback>,
        timeout: Duration,
    ) -> LoginResult;
}

pub trait UsernameAndCodeLoginMethodCallback: Send + Sync {
    fn on_code(&self, url: Option<String>) -> Result<VerifyCode, CoreError>;
    fn clone_box(&self) -> Box<dyn UsernameAndCodeLoginMethodCallback>;
}

pub enum VerifyCode {
    Simple(String),
    WithCaptcha {
        code: String,
        captcha_data: serde_json::Value,
    },
}
