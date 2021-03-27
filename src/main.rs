use std::process::exit;
use docopt::Docopt;
use serde::Deserialize;

mod slack_api;
mod settings;

const USAGE: &str = "
Set Slack status message/emoji/expiration. Edit ~/.slack_status.toml to configure.

Usage:
  slack_status [--get] <status>
  slack_status --list
  slack_status --version
  slack_status (-h | --help)

Options:
  -h --help     Show this screen.
  -l --list     List available statuses
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_status: String,
    flag_get: bool,
    flag_list: bool,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.version(Some(String::from(VERSION))).deserialize())
        .unwrap_or_else(|e| e.exit());

    let app_settings = settings::AppSettings::new().unwrap_or_else(|e| {
        eprintln!("Configuration file error: {}.", e);
        exit(1)
    });

    if args.flag_list {
        println!("Supported statuses: {:?}", app_settings.status.keys());
        exit(0);
    }

    let status = app_settings
        .status
        .get(&args.arg_status)
        .unwrap_or_else(|| {
            eprintln!(
                "Error: cannot find status {:?} in the configuration file.",
                &args.arg_status
            );
            exit(1)
        });

    if args.flag_get {
        let (account, token) = app_settings.accounts.into_iter().next().unwrap_or_else(|| {
            println!("No accounts are defined!");
            exit(2);
        });

        print!("Getting status for account {:?}: ", account);
        exit(match slack_api::get_status(&token) {
            Ok(real_status) => {
                println!("{}", real_status);
                if status.text == real_status {
                    0
                } else {
                    1
                }
            }
            Err(e) => {
                println!("Slack API error: {}", e);
                2
            }
        });
    } else {
        let mut error_occurred = false;
        for (account, token) in app_settings.accounts {
            print!(
                "Setting status {:?} for account {:?}: ",
                status.text, account
            );
            match slack_api::set_status(&token, status) {
                Ok(_profile) => println!("OK"),
                Err(e) => {
                    println!("Slack API error: {}", e);
                    error_occurred = true;
                }
            };
        }
        exit(if error_occurred { 1 } else { 0 });
    }
}
