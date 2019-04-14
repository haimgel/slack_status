extern crate slack_api as slack;

use std::error::Error;
use std::process::exit;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use docopt::Docopt;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
struct SlackStatus {
    emoji: String,
    text: String,
    duration: Option<u64>
}

#[derive(Debug)]
struct AppSettings {
    status: SlackStatus,
    accounts: HashMap<String, String>
}

fn set_status(token :&str, status :&SlackStatus) ->
        Result<slack::UserProfile, Box<Error>> {
    let client = slack::default_client()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiration = match status.duration {
        Some(duration) => now + duration * 60,
        None => 0
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
    let response = slack::users_profile::set(
        &client, &token, &request);
    Ok(response?.profile.unwrap())
}

fn read_settings(status_name :&str) -> Result<AppSettings, Box<Error>> {
    let mut cfg = config::Config::default();
    cfg
        .merge(config::File::with_name("settings"))?
        .merge(config::Environment::with_prefix("APP"))?;

    let mut accounts :HashMap<String, String> = HashMap::new();
    for (acc_name, token_value) in cfg.get_table("accounts")? {
        accounts.insert(acc_name, token_value.into_str()?);
    }

    let status_table = cfg
        .get_table("status")?
        .get(status_name)
        .ok_or("Status not found in the configuration")?
        .clone().into_table()?;
    let emoji = status_table
        .get("emoji")
        .ok_or("Status emoji is required")?
        .clone().into_str()?;
    let text = status_table
        .get("text")
        .ok_or("Status text is required")?
        .clone().into_str()?;
    let duration = if status_table.contains_key("duration") {
        Some(status_table.get("duration").unwrap().clone().into_int()? as u64)
    } else {
        None
    };
    let status = SlackStatus { emoji, text, duration };
    Ok(AppSettings { accounts, status })
}

const USAGE: &'static str = "
Set Slack status message/emoji/expiration. Edit settings.toml to configure.

Usage:
  ss <status>
  ss (-h | --help)
  ss --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_status: String,
    flag_version: bool,
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        print!("slack_status (ss) v{:?}", VERSION);
        exit(1);
    }

    let app_settings = read_settings(&args.arg_status).unwrap();

    for (account, token) in app_settings.accounts {
        print!("Setting status {:?} for account {:?}: ",
                 app_settings.status.text, account);
        match set_status(&token, &app_settings.status) {
            Ok(_profile) => println!("OK"),
            Err(e) => println!("Error: {:?}", e)
        }
    }
}
