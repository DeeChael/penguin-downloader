use std::sync::Arc;
use std::time::Duration;

use crate::error::CoreError;

/// 登录结果类型。
pub type LoginResult = Result<String, CoreError>;

/// 音源提供者支持的登录方式。
pub enum LoginMethod {
    /// 二维码登录。
    Qr {
        /// 登录方式 ID。
        id: String,
        /// 二维码登录处理器。
        handler: Arc<dyn QrLoginMethodHandler>,
    },
    /// 打开 URL 登录。
    Url {
        /// 登录方式 ID。
        id: String,
        /// URL 登录处理器。
        handler: Arc<dyn UrlLoginMethodHandler>,
    },
    /// 通过 Token 登录。
    Token {
        /// 登录方式 ID。
        id: String,
        /// Token 登录处理器。
        handler: Arc<dyn TokenLoginMethodHandler>,
    },
    /// 通过用户名和密码登录。
    UsernameAndPassword {
        /// 登录方式 ID。
        id: String,
        /// 用户名密码登录处理器。
        handler: Arc<dyn UsernameAndPasswordLoginMethodHandler>,
    },
    /// 通过用户名和验证码登录。
    UsernameAndCode {
        /// 登录方式 ID。
        id: String,
        /// 再次发送验证码的间隔（秒），0 表示可直接重新发送。
        resend_after: u64,
        /// 验证码登录处理器。
        handler: Arc<dyn UsernameAndCodeLoginMethodHandler>,
    },
}

/// 二维码登录处理器。
pub trait QrLoginMethodHandler: Send + Sync {
    /// 开始二维码登录流程。
    ///
    /// 调用此函数并传入回调接收二维码，函数会阻塞线程直到登录完成或超时。
    /// 返回登录凭证字符串，失败时返回错误。
    fn login(&self, callback: Box<dyn QrLoginMethodCallback>, timeout: Duration) -> LoginResult;
}

/// 二维码登录回调，用于接收二维码数据。
pub trait QrLoginMethodCallback: Send + Sync {
    /// 收到二维码数据时调用。
    fn on_qr_data(&self, data: QrLoginData);
    /// 克隆此回调对象。
    fn clone_box(&self) -> Box<dyn QrLoginMethodCallback>;
}

/// 二维码数据。
pub enum QrLoginData {
    /// 二维码图片的 bitmap 数据。
    Image(Vec<u8>),
    /// 指向二维码图片的 URL。
    Url(String),
}

/// URL 登录处理器。
pub trait UrlLoginMethodHandler: Send + Sync {
    /// 开始 URL 登录流程。
    ///
    /// 调用此函数并传入回调接收 URL，函数会阻塞线程直到登录完成或超时。
    /// 返回登录凭证字符串，失败时返回错误。
    fn login(&self, callback: Box<dyn UrlLoginMethodCallback>, timeout: Duration) -> LoginResult;
}

/// URL 登录回调，用于接收登录 URL。
pub trait UrlLoginMethodCallback: Send + Sync {
    /// 收到登录 URL 时调用。
    fn on_url(&self, url: String);
    /// 克隆此回调对象。
    fn clone_box(&self) -> Box<dyn UrlLoginMethodCallback>;
}

/// Token 登录处理器。
pub trait TokenLoginMethodHandler: Send + Sync {
    /// 使用 Token 登录。
    ///
    /// 传入 token 字符串，函数会阻塞线程直到登录完成或超时。
    /// 返回登录凭证字符串，失败时返回错误。
    fn login(&self, token: &str, timeout: Duration) -> LoginResult;
}

/// 用户名密码登录处理器。
pub trait UsernameAndPasswordLoginMethodHandler: Send + Sync {
    /// 使用用户名和密码登录。
    ///
    /// 传入用户名、密码和回调，函数会阻塞线程直到登录完成或超时。
    /// 返回登录凭证字符串，失败时返回错误。
    fn login(
        &self,
        username: &str,
        password: &str,
        callback: Box<dyn UsernameAndPasswordLoginMethodCallback>,
        timeout: Duration,
    ) -> LoginResult;
}

/// 用户名密码登录回调，用于处理人机验证。
pub trait UsernameAndPasswordLoginMethodCallback: Send + Sync {
    /// 收到人机验证请求时调用。
    ///
    /// `url` 为人机验证页面的 URL，如果为 `None` 则无需人机验证。
    /// 返回人机验证数据。
    fn on_captcha(&self, url: Option<String>) -> Option<serde_json::Value>;
    /// 克隆此回调对象。
    fn clone_box(&self) -> Box<dyn UsernameAndPasswordLoginMethodCallback>;
}

/// 用户名验证码登录处理器。
pub trait UsernameAndCodeLoginMethodHandler: Send + Sync {
    /// 使用用户名和验证码登录。
    ///
    /// 传入用户名和回调，函数会阻塞线程直到登录完成或超时。
    /// 返回登录凭证字符串，失败时返回错误。
    fn login(
        &self,
        username: &str,
        callback: Box<dyn UsernameAndCodeLoginMethodCallback>,
        timeout: Duration,
    ) -> LoginResult;
}

/// 用户名验证码登录回调，用于接收验证码和处理人机验证。
pub trait UsernameAndCodeLoginMethodCallback: Send + Sync {
    /// 在调用完 UsernameAndCodeLoginMethodHandler 的 login 函数并请求完验证码时调用。
    ///
    /// `url` 为人机验证页面的 URL，如果为 `None` 则无需人机验证。
    /// 需要阻塞等待用户输入验证码，然后返回 `VerifyCode`。
    fn on_code(&self, url: Option<String>) -> Result<VerifyCode, CoreError>;
    /// 克隆此回调对象。
    fn clone_box(&self) -> Box<dyn UsernameAndCodeLoginMethodCallback>;
}

/// 验证码数据。
pub enum VerifyCode {
    /// 普通验证码。
    Simple(String),
    /// 带有人机验证信息的验证码。
    WithCaptcha {
        /// 验证码。
        code: String,
        /// 人机验证数据。
        captcha_data: serde_json::Value,
    },
}
