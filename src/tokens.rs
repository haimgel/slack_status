use anyhow::{Error, Result};
use get_cookie::get_cookie;
use regex::Regex;
use std::sync::Arc;
use url::{ParseError, Url};

pub struct TokenAndCookie {
    pub token: String,
    pub d_cookie: Option<String>,
}

/// Resolve a token or Slack team URL to an actual token
pub fn resolve(token_or_url: &str) -> Result<TokenAndCookie> {
    if token_or_url.starts_with("https://") {
        token_for_team(token_or_url)
    } else {
        Ok(TokenAndCookie {
            token: String::from(token_or_url),
            d_cookie: None,
        })
    }
}

/// Get a token from a team URL (using local browsers' cookies)
fn token_for_team(team_url: &str) -> Result<TokenAndCookie> {
    let cookie_str = get_cookie(".slack.com", "d")?;

    let jar = reqwest::cookie::Jar::default();
    jar.add_cookie_str(
        &format!("d={}; Domain=.slack.com", cookie_str),
        &Url::parse("https://slack.com/").unwrap(),
    );

    // Slack _needs_ user agent pretending to be Chrome, otherwise it does not give back a token
    let client = reqwest::blocking::Client::builder()
        .cookie_provider(Arc::new(jar))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36")
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

    Ok(TokenAndCookie {
        token: String::from(find.as_str()),
        d_cookie: Some(cookie_str),
    })
}
