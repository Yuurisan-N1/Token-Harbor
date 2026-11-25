use std::path::Path;
use std::time::Duration;

use crate::job::onboard;
use crate::support::cfg::loader;
use crate::support::consts::app::STEP_RETRY_MILLIS;
use crate::support::error::AppResult;
use crate::support::log::format::{countdown, gap, lg, ly};

pub async fn run(root: &Path, target: usize) -> AppResult<()> {
    cycle(root, target, false).await
}

pub async fn enroll(root: &Path, target: usize) -> AppResult<()> {
    cycle(root, target, true).await
}

async fn cycle(root: &Path, target: usize, fresh: bool) -> AppResult<()> {
    let setup = loader::load(root)?;
    let target = target.max(1);
    let mut made = 0usize;
    let mut attempt = 0usize;

    while made < target {
        lg(&format!("Starting automation cycle number {}", attempt + 1));
        match once(&setup, attempt, fresh).await {
            Ok(()) => {
                made += 1;
                lg(&format!(
                    "Automation cycle number {} is complete",
                    attempt + 1
                ));
            }
            Err(err) => {
                ly(&format!(
                    "Automation cycle number {} stopped: {}",
                    attempt + 1,
                    err
                ));
                tokio::time::sleep(Duration::from_millis(STEP_RETRY_MILLIS)).await;
            }
        }
        attempt += 1;
        gap();
    }

    countdown(setup.config.sleep_seconds(), "Next cycle starts in").await;
    Ok(())
}

async fn once(
    setup: &crate::support::cfg::loader::Setup,
    index: usize,
    fresh: bool,
) -> AppResult<()> {
    if fresh {
        onboard::signup(setup, index).await?;
        return Ok(());
    }
    match onboard::stored_key(setup).ok() {
        Some(key) => {
            let account = onboard::Account {
                email: onboard::stored_email(setup)?.unwrap_or_default(),
                password: String::new(),
                api_key: key,
                device: crate::support::device::Device::plain(),
            };
            onboard::drive(setup, &account, index).await?;
        }
        None => {
            onboard::signup(setup, index).await?;
        }
    }
    Ok(())
}
