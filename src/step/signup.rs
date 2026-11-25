use rand::rngs::OsRng;
use rand::RngCore;

use crate::net::client::{Reply, Site};
use crate::support::consts::app::{EMAIL_RANDOM_LEN, PASSWORD_MIN_LEN};
use crate::support::consts::net::{PRECHECK_PATH, ROUTER_TREE, SIGNUP_PAGE};
use crate::support::device::Device;
use crate::support::error::{refusal, AppError, AppResult};
use crate::wire::form::Form;
use crate::wire::shape::rsc_refusal;

pub struct Identity {
    pub fingerprint: String,
    pub password: String,
    pub device: Device,
}

pub struct Envelope {
    pub action_ref: String,
    pub action_id: String,
    pub action_args: String,
    pub action_key: String,
}

pub fn fresh_identity() -> Identity {
    Identity {
        fingerprint: uuid(),
        password: format!("TH-{}-{}!", hex(4), hex(4)),
        device: Device::fresh(),
    }
}

fn pull(html: &str, name: &str) -> Option<String> {
    let needle = format!("name=\"{}\" value=\"", name);
    let start = html.find(&needle)? + needle.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    Some(unescape(&rest[..end]))
}

fn unescape(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

pub fn envelope(page: &str) -> AppResult<Envelope> {
    let action_id = pull(page, "$ACTION_1:0")
        .ok_or_else(|| AppError::api("the signup page carried no action id"))?;
    let action_key = pull(page, "$ACTION_KEY")
        .ok_or_else(|| AppError::api("the signup page carried no action key"))?;
    let action_args = pull(page, "$ACTION_1:1").unwrap_or_else(|| "[\"$undefined\"]".to_string());
    Ok(Envelope {
        action_ref: String::new(),
        action_id,
        action_args,
        action_key,
    })
}

pub async fn submit(
    site: &Site,
    token: &str,
    email: &str,
    timezone: &str,
    identity: &Identity,
) -> AppResult<()> {
    if token.chars().count() < PASSWORD_MIN_LEN {
        return Err(AppError::network("the solver returned a short token"));
    }

    let page = site.get_with(SIGNUP_PAGE, &[("accept", "text/html".to_string())]).await?;
    if !page.is_success() {
        return Err(AppError::api("the signup page did not load"));
    }
    let envelope = envelope(&page.text)?;

    let probe = format!("{}?fp={}", PRECHECK_PATH, identity.fingerprint);
    let _ = site.get_with(&probe, &[("accept", "*/*".to_string())]).await;

    let mut form = Form::new();
    form.field("$ACTION_REF_1", &envelope.action_ref)
        .field("$ACTION_1:0", &envelope.action_id)
        .field("$ACTION_1:1", &envelope.action_args)
        .field("$ACTION_KEY", &envelope.action_key)
        .field("device_fingerprint", &identity.fingerprint)
        .field("timezone", timezone)
        .field("age_confirmed", "1")
        .field("next", "")
        .field("cf-turnstile-response", token)
        .field("email", email)
        .field("password", &identity.password)
        .field("invite_code", "");
    let (content_type, body) = form.finish();

    let extra: Vec<(&str, String)> = vec![
        ("next-router-state-tree", ROUTER_TREE.to_string()),
        ("origin", crate::support::consts::net::BASE_URL.to_string()),
        (
            "referer",
            format!("{}{}", crate::support::consts::net::BASE_URL, SIGNUP_PAGE),
        ),
        ("user-agent", identity.device.agent.clone()),
    ];

    let reply = site
        .post_rsc(SIGNUP_PAGE, &content_type, body, &extra)
        .await?;
    judge(&reply)
}

fn judge(reply: &Reply) -> AppResult<()> {
    if reply.status == 400 {
        return Err(AppError::api("the server action envelope was refused"));
    }
    if let Some(text) = rsc_refusal(&reply.text) {
        return Err(AppError::api(refusal(&text)));
    }
    if reply.status >= 500 {
        return Err(AppError::api(
            "the server action threw while creating the account",
        ));
    }
    Ok(())
}

pub fn hex(len: usize) -> String {
    let mut raw = vec![0u8; len];
    OsRng.fill_bytes(&mut raw);
    raw.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn uuid() -> String {
    let mut raw = [0u8; 16];
    OsRng.fill_bytes(&mut raw);
    raw[6] = (raw[6] & 0x0f) | 0x40;
    raw[8] = (raw[8] & 0x3f) | 0x80;
    let body: String = raw.iter().map(|byte| format!("{:02x}", byte)).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &body[0..8],
        &body[8..12],
        &body[12..16],
        &body[16..20],
        &body[20..32]
    )
}

pub fn fresh_email(domain: &str) -> String {
    format!(
        "{}{}@{}",
        crate::support::consts::app::EMAIL_PREFIX,
        hex(EMAIL_RANDOM_LEN / 2),
        domain
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_uuid_has_the_expected_shape() {
        let text = uuid();
        let parts: Vec<&str> = text.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn the_envelope_is_pulled_from_the_hidden_inputs() {
        let page = "<input type=\"hidden\" name=\"$ACTION_1:0\" value=\"{&quot;id&quot;:&quot;abc&quot;,&quot;bound&quot;:&quot;$@1&quot;}\"/><input type=\"hidden\" name=\"$ACTION_1:1\" value=\"[&quot;$undefined&quot;]\"/><input type=\"hidden\" name=\"$ACTION_KEY\" value=\"k123\"/>";
        let found = envelope(page).unwrap();
        assert_eq!(found.action_id, "{\"id\":\"abc\",\"bound\":\"$@1\"}");
        assert_eq!(found.action_args, "[\"$undefined\"]");
        assert_eq!(found.action_key, "k123");
    }

    #[test]
    fn a_page_without_the_action_id_is_refused() {
        let page = "<input type=\"hidden\" name=\"$ACTION_KEY\" value=\"k123\"/>";
        assert!(envelope(page).is_err());
    }
}
