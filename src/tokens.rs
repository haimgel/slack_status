use anyhow::{Error, Result};
use get_cookie::get_cookie;
use regex::Regex;
use std::sync::Arc;
use url::{ParseError, Url};

/// Resolve a token or Slack team URL to an actual token
pub fn resolve(token_or_url: &str) -> Result<String> {
    if token_or_url.starts_with("https://") {
        token_for_team(token_or_url)
    } else {
        Ok(String::from(token_or_url))
    }
}

/// Get a token from a team URL (using local browsers' cookies)
fn token_for_team(team_url: &str) -> Result<String> {
    let cookie_str = get_cookie(".slack.com", "d")?;
    //let cookie_str = "DIwR2ESX2yAcDrgRYcTJVkekoyuWN4lfOPXnrds4TRvp%2B6NnmV8GZSIjdZMZuP1RVXIk04RUE46IepbmoAz2nM0Yw9BwcoG3lqp%2FZ6zn62M5WCUWI2HCppvqSLNNHpzgla4yEive2gJ5VcBGbncZSmqDlbUmckm72jyejHrl6uxKz4RO5xeQL3Be";

    let jar = reqwest::cookie::Jar::default();
    jar.add_cookie_str(
        &format!("d={}; Domain=.slack.com", cookie_str),
        &Url::parse("https://slack.com/").unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .cookie_provider(Arc::new(jar))
        .user_agent(format!("slack_status/{}", env!("CARGO_PKG_VERSION")))
        /*
        // To test requests and responses in Charles Proxy
        .danger_accept_invalid_certs(true)
        .proxy(reqwest::Proxy::https("http://localhost:8888")?)
        */
        .build()?;

    let team_url = Url::parse(team_url)?;
    let domain = team_url.host().ok_or(ParseError::EmptyHost)?;
    let scrape_url = format!("https://{}/customize/emoji", domain);

    let response = client.get(scrape_url).send()?.text()?;

    let token_regex = Regex::new(r#""api_token":"(xo[^"]+)""#).unwrap();
    let find = token_regex
        .captures(&response)
        .and_then(|captures| captures.get(1))
        .ok_or_else(|| Error::msg("No token in the scraped page"))?;

    Ok(String::from(find.as_str()))
}
