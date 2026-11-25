use std::sync::Arc;
use std::time::Duration;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Method, Url};
use serde_json::Value;

use crate::net::egress::Proxy;
use crate::support::consts::app::AUTH_COOKIE_MARK;
use crate::support::consts::net::{BASE_URL, CONNECT_TIMEOUT_SECS, READ_TIMEOUT_SECS};
use crate::support::device::Device;
use crate::support::error::{AppError, AppResult};

pub struct Reply {
    pub status: u16,
    pub text: String,
    #[allow(dead_code)]
    pub location: Option<String>,
    pub headers: Vec<(String, String)>,
}

impl Reply {
    pub fn header(&self, name: &str) -> Option<&str> {
        let wanted = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| key.to_ascii_lowercase() == wanted)
            .map(|(_, value)| value.as_str())
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

pub struct Site {
    http: Client,
    jar: Arc<Jar>,
    base: String,
    bearer: String,
    device: Device,
}

impl Site {
    pub fn open(base: &str, proxy: Option<&Proxy>) -> AppResult<Site> {
        Site::open_as(base, proxy, Device::plain())
    }

    pub fn open_as(base: &str, proxy: Option<&Proxy>, device: Device) -> AppResult<Site> {
        let jar = Arc::new(Jar::default());
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(READ_TIMEOUT_SECS))
            .user_agent(device.agent.clone())
            .cookie_provider(jar.clone())
            .gzip(true);
        if let Some(proxy) = proxy {
            let route = reqwest::Proxy::all(proxy.url())
                .map_err(|_| AppError::config("the proxy could not be used"))?;
            builder = builder.proxy(route);
        }
        let http = builder
            .build()
            .map_err(|_| AppError::network("the client could not be built"))?;
        Ok(Site {
            http,
            jar,
            base: base.trim_end_matches('/').to_string(),
            bearer: String::new(),
            device,
        })
    }

    pub fn adopt(&mut self, key: &str) {
        self.bearer = key.trim().to_string();
    }

    #[allow(dead_code)]
    pub fn has_bearer(&self) -> bool {
        !self.bearer.is_empty()
    }

    pub fn has_auth_cookie(&self) -> bool {
        let url = match Url::parse(&self.base) {
            Ok(url) => url,
            Err(_) => return false,
        };
        match self.jar.cookies(&url) {
            Some(header) => header
                .to_str()
                .map(|text| text.contains(AUTH_COOKIE_MARK))
                .unwrap_or(false),
            None => false,
        }
    }

    pub async fn get(&self, path: &str) -> AppResult<Reply> {
        self.send(Method::GET, path, None, None, &[]).await
    }

    pub async fn get_with(&self, path: &str, extra: &[(&str, String)]) -> AppResult<Reply> {
        self.send(Method::GET, path, None, None, extra).await
    }

    #[allow(dead_code)]
    pub async fn get_rsc(&self, path: &str, extra: &[(&str, String)]) -> AppResult<Reply> {
        self.send(Method::GET, path, None, Some("text/x-component"), extra)
            .await
    }

    pub async fn post_rsc(
        &self,
        path: &str,
        content_type: &str,
        body: Vec<u8>,
        extra: &[(&str, String)],
    ) -> AppResult<Reply> {
        self.send(
            Method::POST,
            path,
            Some((content_type, body)),
            Some("text/x-component"),
            extra,
        )
        .await
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> AppResult<Reply> {
        let text = serde_json::to_string(body)
            .map_err(|_| AppError::config("the request body could not be written"))?;
        self.send(
            Method::POST,
            path,
            Some(("application/json", text.into_bytes())),
            None,
            &[],
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn post_form(
        &self,
        path: &str,
        content_type: &str,
        body: Vec<u8>,
        extra: &[(&str, String)],
    ) -> AppResult<Reply> {
        self.send(Method::POST, path, Some((content_type, body)), None, extra)
            .await
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<(&str, Vec<u8>)>,
        accept: Option<&str>,
        extra: &[(&str, String)],
    ) -> AppResult<Reply> {
        let url = format!("{}{}", self.base, path);
        let mut request = self
            .http
            .request(method, &url)
            .header("accept-language", self.device.language.as_str())
            .header("sec-ch-ua", self.device.brands.as_str())
            .header("sec-ch-ua-mobile", self.device.mobile.as_str())
            .header("sec-ch-ua-platform", self.device.platform.as_str());

        if let Some(accept) = accept {
            request = request.header("accept", accept);
        }
        if let Some((content_type, payload)) = body {
            request = request.header("content-type", content_type).body(payload);
        }
        if !self.bearer.is_empty() {
            request = request.header("authorization", format!("Bearer {}", self.bearer));
        }
        for (name, value) in extra {
            request = request.header(*name, value.as_str());
        }

        let reply = request.send().await?;
        let status = reply.status().as_u16();
        let location = reply
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let headers: Vec<(String, String)> = reply
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();
        let text = reply.text().await?;
        Ok(Reply {
            status,
            text,
            location,
            headers,
        })
    }
}

#[allow(dead_code)]
pub fn bearer_base(base: &str) -> String {
    base.trim_end_matches('/').to_string()
}

#[allow(dead_code)]
pub fn site() -> &'static str {
    BASE_URL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reply_reads_its_headers_without_case() {
        let reply = Reply {
            status: 200,
            text: String::new(),
            #[allow(dead_code)]
            location: None,
            headers: vec![("X-Th-Plan".to_string(), "free".to_string())],
        };
        assert_eq!(reply.header("x-th-plan"), Some("free"));
        assert!(reply.is_success());
    }

    #[test]
    fn the_base_never_keeps_a_trailing_slash() {
        assert_eq!(bearer_base("https://a.test/"), "https://a.test");
    }

    #[test]
    fn a_fresh_site_has_no_bearer_and_no_cookie() {
        let site = Site::open(BASE_URL, None).unwrap();
        assert!(!site.has_bearer());
        assert!(!site.has_auth_cookie());
    }
}
