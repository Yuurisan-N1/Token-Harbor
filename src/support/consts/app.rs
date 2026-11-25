pub const CONFIG_FILE: &str = "config.json";
pub const DATA_FILE: &str = "data.txt";
pub const PROXY_FILE: &str = "proxy.txt";
pub const STATE_FILE: &str = "session.json";

#[allow(dead_code)]
pub const ENABLE_FREE_MODELS: bool = true;

pub const STEP_ATTEMPTS: u32 = 3;
pub const STEP_RETRY_MILLIS: u64 = 2000;

pub const MAIL_POLLS: u32 = 40;
pub const MAIL_POLL_SECONDS: u64 = 5;
pub const MAIL_RETRY_SECONDS: u64 = 20;

pub const EMAIL_PREFIX: &str = "th-";
pub const EMAIL_RANDOM_LEN: usize = 12;
pub const PASSWORD_MIN_LEN: usize = 12;

pub const KEY_PREFIX: &str = "thk_live_";
pub const KEY_NAME: &str = "tokenharbor";
pub const AUTH_COOKIE_MARK: &str = "sb-auth-auth-token";
