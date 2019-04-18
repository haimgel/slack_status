extern crate slack_api as slack;

use std::collections::HashMap;
use std::error::Error;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};

use dirs::home_dir;
use docopt::Docopt;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct SlackStatus {
    emoji: String,
    text: String,
    duration: Option<u64>
}

#[derive(Debug, Deserialize)]
struct AppSettings {
    status: HashMap<String, SlackStatus>,
    accounts: HashMap<String, String>
}

impl AppSettings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut settings_path = home_dir().unwrap();
        settings_path.push(".slack_status");
        let mut cfg = config::Config::default();
        cfg
            .merge(config::File::from(settings_path))?
            .merge(config::Environment::with_prefix("APP"))?;
        cfg.try_into()
    }
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

const USAGE: &'static str = "
Set Slack status message/emoji/expiration. Edit settings.toml to configure.

Usage:
  slack_status <status>
  slack_status (-h | --help)
  slack_status --list
  slack_status --version

Options:
  -h --help     Show this screen.
  -l --list     List available statuses
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_status: String,
    flag_version: bool,
    flag_list: bool
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    if args.flag_version {
        println!("slack_status v{}", VERSION);
        exit(0);
    }

    let app_settings = AppSettings::new().unwrap_or_else( |e| {
        eprintln!("Configuration file error: {}.", e);
        exit(1)
    });

    if args.flag_list {
        println!("Supported statuses: {:?}", app_settings.status.keys());
        exit(0)
    }

    let status = app_settings.status.get(&args.arg_status).unwrap_or_else( ||{
        eprintln!("Error: cannot find status {:?} in the configuration file.", &args.arg_status);
        exit(1)
    });

    for (account, token) in app_settings.accounts {
        print!("Setting status {:?} for account {:?}: ",
                 status.text, account);
        match set_status(&token, status) {
            Ok(_profile) => println!("OK"),
            Err(e) => println!("Slack API error: {}", e)
        }
    }
}
