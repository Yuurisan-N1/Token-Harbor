use std::io::{IsTerminal, Write};
use std::time::Duration;

use super::init::{blank, clear_live, emit, finish_live, live, Z};

pub const G: &str = "\x1b[1;32m";
pub const Y: &str = "\x1b[1;33m";
pub const R: &str = "\x1b[1;31m";

pub fn lg(msg: &str) {
    emit(G, msg);
}

pub fn ly(msg: &str) {
    emit(Y, msg);
}

pub fn lr(msg: &str) {
    emit(R, msg);
}

pub fn gap() {
    blank();
}

pub fn status(msg: &str) {
    live(Y, msg);
}

pub fn clear_status() {
    clear_live();
}

pub fn finish_status() {
    finish_live();
}

pub fn squeeze(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut sp = false;
    for c in input.chars() {
        if c.is_control() || c == '\u{feff}' {
            if c == '\n' || c == '\r' || c == '\t' {
                sp = true;
            }
            continue;
        }
        if c.is_whitespace() {
            sp = true;
            continue;
        }
        if sp && !out.is_empty() {
            out.push(' ');
        }
        sp = false;
        out.push(c);
    }
    out
}

pub fn clean_text(value: &str, fallback: &str) -> String {
    let mut text = String::with_capacity(value.len());
    for c in value.chars() {
        if "[]|#!@$%^&*()-".contains(c) {
            text.push(' ');
        } else {
            text.push(c);
        }
    }
    let text = squeeze(&text);
    if text.is_empty() {
        fallback.to_string()
    } else {
        text
    }
}

pub fn clock(seconds: u64) -> String {
    let days = seconds / 86_400;
    let rest = seconds % 86_400;
    let hours = rest / 3_600;
    let minutes = (rest % 3_600) / 60;
    let secs = rest % 60;
    if days > 0 {
        format!("{}d {:02}:{:02}:{:02}", days, hours, minutes, secs)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }
}

pub fn num(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    let text = format!("{:.4}", value);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-" {
        return "0".to_string();
    }
    text.to_string()
}

pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((head, domain)) => {
            let keep: String = head.chars().take(2).collect();
            let tail: String = "*".repeat(head.chars().count().saturating_sub(2).max(3));
            format!("{}{}@{}", keep, tail, domain)
        }
        None => "***".to_string(),
    }
}

pub fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 18 {
        return "**".to_string();
    }
    let head: String = chars[..8].iter().collect();
    let tail: String = chars[chars.len() - 10..].iter().collect();
    format!("{}**{}", head, tail)
}

#[allow(dead_code)]
pub fn ask(prompt: &str) -> String {
    let mut line = String::new();
    print!("{}{}{} ", Y, squeeze(prompt), Z);
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut line);
    line.trim().to_string()
}

pub async fn countdown(seconds: u64, label: &str) {
    if seconds == 0 {
        return;
    }
    let text = clean_text(label, "Next cycle starts in");
    if !std::io::stdout().is_terminal() {
        ly(&format!("{} {}", text, clock(seconds)));
        tokio::time::sleep(Duration::from_secs(seconds)).await;
        return;
    }
    let mut left = seconds;
    while left > 0 {
        status(&format!("{} {}", text, clock(left)));
        tokio::time::sleep(Duration::from_secs(1)).await;
        left -= 1;
    }
    clear_status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_drops_banned_symbols() {
        assert_eq!(clean_text("Ad Task #1", "task"), "Ad Task 1");
        assert_eq!(clean_text("###", "task 7"), "task 7");
    }

    #[test]
    fn squeeze_drops_control_chars() {
        assert_eq!(squeeze("  task\tlist\nok  "), "task list ok");
    }

    #[test]
    fn clock_uses_two_shapes() {
        assert_eq!(clock(3_723), "01:02:03");
        assert_eq!(clock(90_061), "1d 01:01:01");
    }

    #[test]
    fn num_trims_the_tail_like_the_canon() {
        assert_eq!(num(0.0), "0");
        assert_eq!(num(12.5), "12.5");
        assert_eq!(num(1.234567), "1.2346");
        assert_eq!(num(f64::NAN), "0");
    }

    #[test]
    fn the_email_is_masked_at_the_head() {
        let text = mask_email("th-abc123def456@euploading.com");
        assert!(text.starts_with("th"));
        assert!(text.ends_with("@euploading.com"));
        assert!(!text.contains("abc123def456"));
    }

    #[test]
    fn the_key_keeps_a_head_and_a_tail_only() {
        let text =
            mask_key("thk_live_yOz0d7fN27UC6jglOX73DOfSau93Qo2-n9wKjyzxyoeV7ALLP0Nhpv_R940E_vEX");
        assert_eq!(text, "thk_live**_R940E_vEX");
        assert!(!text.contains("yOz0d7fN"));
    }

    #[test]
    fn a_short_key_is_masked_whole() {
        assert_eq!(mask_key("thk_live_abc"), "**");
    }

    #[test]
    fn the_ansi_reset_is_the_canon_one() {
        assert_eq!(crate::support::log::init::Z, "\x1b[0m");
    }
}
