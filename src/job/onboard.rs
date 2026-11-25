use std::future::Future;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::net::client::Site;
use crate::net::gate::{read_settings, Gate};
use crate::step::mailbox::Mailbox;
use crate::step::{apikey, freemode, signup, verify};
use crate::support::cfg::loader::Setup;
use crate::support::consts::app::{
    DATA_FILE, KEY_NAME, KEY_PREFIX, STATE_FILE, STEP_ATTEMPTS, STEP_RETRY_MILLIS,
};
use crate::support::consts::net::{
    BASE_URL, SOLVER_FILE, SOLVER_POLLS, SOLVER_RETRY_MILLIS, SOLVER_TOKEN_ATTEMPTS,
};
use crate::support::device::Device;
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::{clock, lg, ly, mask_email, mask_key, num, status};
use crate::wire::shape::{model_ids, plaintext_key};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub password: String,
    pub api_key: String,
    pub created_at: String,
    #[serde(default)]
    pub device: Option<Device>,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub email: String,
    pub password: String,
    pub api_key: String,
    pub device: Device,
}

pub async fn signup(setup: &Setup, slot: usize) -> AppResult<Account> {
    let proxy = setup.proxy_for(slot);

    lg(&format!("Talking to {}", BASE_URL));
    if let Some(proxy) = proxy {
        ly(&format!("Using proxy {}", proxy.masked()));
    }

    let identity = signup::fresh_identity();
    let site = Site::open_as(BASE_URL, proxy, identity.device.clone())?;

    status("Opening the mailbox");
    let mailbox = Mailbox::open(proxy).await?;
    lg(&format!(
        "The inbox is open for {}",
        mask_email(mailbox.address())
    ));

    status("Creating the account");
    let token = fetch_token(setup).await?;
    signup::submit(
        &site,
        &token,
        mailbox.address(),
        &setup.config.timezone(),
        &identity,
    )
    .await?;
    if !site.has_auth_cookie() {
        return Err(AppError::api(
            "the server did not hand over a session cookie",
        ));
    }
    lg("The account password was accepted by the server");

    status("Waiting for the verification mail");
    let token = again(|| mailbox.wait_quiet()).await?;

    again(|| verify::follow(&site, &token)).await?;
    lg("The account was verified by the server");
    if !verify::confirm(&site).await.unwrap_or(true) {
        ly("The server has not flagged the mail yet");
    }

    again(|| freemode::enable_free_models(&site)).await?;

    status("Creating the key");
    let key = again(|| apikey::create(&site, KEY_NAME)).await?;
    if !key.starts_with(KEY_PREFIX) {
        ly("The key came back without the expected prefix");
    }
    lg(&format!("Key: {}", mask_key(&key)));

    let account = Account {
        email: mailbox.address().to_string(),
        password: identity.password.clone(),
        api_key: key,
        device: identity.device.clone(),
    };
    save(setup, &account)?;
    Ok(account)
}

async fn again<T, F, Fut>(mut step: F) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = AppResult<T>>,
{
    let mut left = STEP_ATTEMPTS.max(1);
    loop {
        match step().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                left -= 1;
                if left == 0 || !worth_again(&err) {
                    return Err(err);
                }
                tokio::time::sleep(Duration::from_millis(STEP_RETRY_MILLIS)).await;
            }
        }
    }
}

fn worth_again(err: &AppError) -> bool {
    matches!(err, AppError::Network(_))
}

async fn fetch_token(setup: &Setup) -> AppResult<String> {
    let per_try = (SOLVER_POLLS / SOLVER_TOKEN_ATTEMPTS).max(1);
    let mut left = SOLVER_TOKEN_ATTEMPTS.max(1);
    loop {
        let outcome = match open_gate(setup.root.as_path()).await {
            Ok(gate) => gate.token(per_try).await,
            Err(err) => Err(err),
        };
        match outcome {
            Ok(token) => return Ok(token),
            Err(err) => {
                left -= 1;
                if left == 0 {
                    return Err(err);
                }
                tokio::time::sleep(Duration::from_millis(SOLVER_RETRY_MILLIS)).await;
            }
        }
    }
}

async fn open_gate(root: &Path) -> AppResult<Gate> {
    let path = root.join(SOLVER_FILE);
    let text = std::fs::read_to_string(&path).map_err(|_| {
        AppError::config(format!(
            "{} is missing; the solver url lives there",
            SOLVER_FILE
        ))
    })?;
    let settings = read_settings(&text);
    Gate::open(&settings.url).await
}

pub fn save(setup: &Setup, account: &Account) -> AppResult<()> {
    let data = setup.root.join(DATA_FILE);
    let mut body = std::fs::read_to_string(&data).unwrap_or_default();
    if !body.is_empty() && !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(&account.api_key);
    body.push('\n');
    std::fs::write(&data, body)
        .map_err(|err| AppError::config(format!("{} could not be written: {}", DATA_FILE, err)))?;

    let state = setup.root.join(STATE_FILE);
    let mut book: serde_json::Map<String, serde_json::Value> = match std::fs::read_to_string(&state)
    {
        Ok(text) => serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default(),
        Err(_) => serde_json::Map::new(),
    };
    book.insert(
        account.email.clone(),
        serde_json::to_value(Entry {
            password: account.password.clone(),
            api_key: account.api_key.clone(),
            created_at: now(),
            device: Some(account.device.clone()),
        })
        .unwrap_or(serde_json::Value::Null),
    );
    std::fs::write(
        &state,
        serde_json::to_string_pretty(&serde_json::Value::Object(book)).unwrap_or_default(),
    )
    .map_err(|err| AppError::config(format!("{} could not be written: {}", STATE_FILE, err)))?;

    lg(&format!(
        "{} holds the key and {} holds the account",
        DATA_FILE, STATE_FILE
    ));
    Ok(())
}

pub async fn models(site: &Site) -> AppResult<Vec<String>> {
    let reply = site.get(crate::wire::endpoint::MODELS).await?;
    if !reply.is_success() {
        return Err(AppError::api(format!(
            "the model list was refused with HTTP {}",
            reply.status
        )));
    }
    let rows = model_ids(&reply.text);
    if rows.is_empty() {
        return Err(AppError::api("the server returned no model"));
    }
    Ok(rows)
}

pub async fn chat(site: &Site, model: &str, prompt: &str) -> AppResult<String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
    });
    let reply = site.post_json(crate::wire::endpoint::CHAT, &body).await?;
    if !reply.is_success() {
        return Err(AppError::api(format!(
            "the chat was refused with HTTP {}",
            reply.status
        )));
    }
    match crate::wire::shape::chat_text(&reply.text) {
        Some(text) => Ok(text),
        None => Err(AppError::api("the reply carried no message")),
    }
}

pub async fn usage(site: &Site) -> AppResult<Vec<(String, String)>> {
    let reply = site.get(crate::wire::endpoint::MODELS).await?;
    if !reply.is_success() {
        return Err(AppError::api(format!(
            "the account was refused with HTTP {}",
            reply.status
        )));
    }
    let mut rows: Vec<(String, String)> = Vec::new();
    if let Some(pct) = reply.header("x-th-free-used-pct") {
        let value = pct.trim().parse::<f64>().unwrap_or(0.0);
        rows.push(("Used".to_string(), format!("{}%", num(value))));
    }
    if let Some(reset) = reply.header("x-th-free-resets") {
        let value = reset.trim().parse::<u64>().unwrap_or(0);
        rows.push(("Resets in".to_string(), clock(value)));
    }
    if rows.is_empty() {
        rows.push((
            "Models".to_string(),
            format!("{} available", model_ids(&reply.text).len()),
        ));
    }
    Ok(rows)
}

pub fn stored_email(setup: &Setup) -> AppResult<Option<String>> {
    let path = setup.root.join(STATE_FILE);
    let body = std::fs::read_to_string(&path).unwrap_or_default();
    let book = serde_json::from_str::<serde_json::Value>(&body).unwrap_or_default();
    Ok(book
        .as_object()
        .and_then(|rows| rows.keys().next().cloned())
        .filter(|value| !value.is_empty()))
}

pub fn stored_device(setup: &Setup, email: &str) -> Option<Device> {
    let path = setup.root.join(STATE_FILE);
    let body = std::fs::read_to_string(&path).ok()?;
    let book: serde_json::Value = serde_json::from_str(&body).ok()?;
    let entry = book.get(email)?;
    serde_json::from_value::<Entry>(entry.clone())
        .ok()
        .and_then(|row| row.device)
}

pub async fn drive(setup: &Setup, account: &Account, slot: usize) -> AppResult<()> {
    let proxy = setup.proxy_for(slot);
    let device = stored_device(setup, &account.email).unwrap_or(account.device.clone());
    let mut site = Site::open_as(BASE_URL, proxy, device)?;
    site.adopt(&account.api_key);

    status("Reading the model list");
    let rows = again(|| models(&site)).await?;
    lg(&format!(
        "The model list answers with {} models",
        rows.len()
    ));

    status("Sending one chat");
    let model = rows
        .iter()
        .find(|id| id.ends_with(":free"))
        .cloned()
        .unwrap_or_else(|| rows[0].clone());
    let text = again(|| chat(&site, &model, "Reply with ok")).await?;
    lg(&format!("{} answered: {}", model, text.trim()));

    status("Reading the usage headers");
    for (name, value) in again(|| usage(&site)).await? {
        ly(&format!("{}: {}", name, value));
    }
    Ok(())
}

pub fn stored_key(setup: &Setup) -> AppResult<String> {
    setup.keys.first().cloned().ok_or_else(|| {
        AppError::config(format!("{} holds no key yet; run signup first", DATA_FILE))
    })
}

#[allow(dead_code)]
pub fn key_of(body: &str) -> Option<String> {
    plaintext_key(body)
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_entry_carries_the_device() {
        let entry = Entry {
            password: "p".to_string(),
            api_key: "thk_live_x".to_string(),
            created_at: "2026-11-20 10:00:00".to_string(),
            device: Some(Device::plain()),
        };
        let value = serde_json::to_value(&entry).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 4);
        assert_eq!(value["api_key"], "thk_live_x");
        assert!(value["device"]["agent"]
            .as_str()
            .unwrap()
            .contains("Mozilla/5.0"));
    }

    #[test]
    fn an_old_entry_without_a_device_still_reads() {
        let text = r#"{"th-a@b.test":{"password":"p","api_key":"k","created_at":"x"}}"#;
        let book: serde_json::Value = serde_json::from_str(text).unwrap();
        let row: Entry = serde_json::from_value(book["th-a@b.test"].clone()).unwrap();
        assert!(row.device.is_none());
    }

    #[tokio::test]
    async fn a_timed_out_step_is_tried_again() {
        let mut calls = 0u32;
        let value = again(|| {
            calls += 1;
            let round = calls;
            async move {
                if round < 3 {
                    Err(AppError::network("the request timed out"))
                } else {
                    Ok(round)
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(value, 3);
        assert_eq!(calls, 3);
    }

    #[tokio::test]
    async fn a_refusal_is_not_tried_again() {
        let mut calls = 0u32;
        let outcome: AppResult<()> = again(|| {
            calls += 1;
            async { Err(AppError::api("the server refused")) }
        })
        .await;
        assert!(outcome.is_err());
        assert_eq!(calls, 1);
    }

    #[test]
    fn the_account_carries_the_three_pieces() {
        let account = Account {
            email: "th-a@b.test".to_string(),
            password: "secret".to_string(),
            api_key: "thk_live_abc".to_string(),
            device: Device::plain(),
        };
        assert_eq!(account.email, "th-a@b.test");
        assert_eq!(account.password, "secret");
    }
}
