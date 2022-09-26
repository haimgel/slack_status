use crate::slack_api_client;
use crate::tokens;
use serde::Deserialize;
use serde_json::json;
use slack_api::sync as slack;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
pub struct SlackStatus {
    pub emoji: String,
    pub text: String,
    pub duration: Option<u64>,
}

pub fn get_status(token_or_url: &str) -> Result<String, Box<dyn Error>> {
    let token_and_cookie = tokens::resolve(&token_or_url)?;
    let client =
        slack_api_client::build_client(token_and_cookie.d_cookie.as_ref().map(String::as_str))?;

    let request = slack::users_profile::GetRequest {
        user: None,
        include_labels: None,
    };
    let response = slack::users_profile::get(&client, &token_and_cookie.token, &request);
    Ok(response?
        .profile
        .ok_or("User profile was not returned")?
        .status_text
        .unwrap_or_default())
}

pub fn set_status(
    token_or_url: &str,
    status: &SlackStatus,
) -> Result<slack::UserProfile, Box<dyn Error>> {
    let token_and_cookie = tokens::resolve(&token_or_url)?;
    let client =
        slack_api_client::build_client(token_and_cookie.d_cookie.as_ref().map(String::as_str))?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiration = match status.duration {
        Some(duration) => now + duration * 60,
        None => 0,
    };

    // See https://api.slack.com/docs/presence-and-status for details about this endpoint
    let profile = json!({
        "status_emoji": status.emoji,
        "status_text": status.text,
        "status_expiration": expiration
    });
    let profile_str = &profile.to_string();
    let request = slack::users_profile::SetRequest {
        user: None,
        profile: Some(profile_str),
        name: None,
        value: None,
    };
    let response = slack::users_profile::set(&client, &token_and_cookie.token, &request);
    Ok(response?.profile.ok_or("User profile was not returned")?)
}
