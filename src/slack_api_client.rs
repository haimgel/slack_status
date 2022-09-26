use reqwest::header;
use slack_api::sync::requests::SlackWebRequestSender;
use std::borrow::Borrow;

pub struct Client {
    inner: reqwest::blocking::Client,
}

// This implementation is nearly the same as in slack_api/src/sync/requests.rs, just that we _have_
// to wrap reqwest::blocking::Client into an outer structure because cannot implement traits for
// types defined elsewhere.
impl SlackWebRequestSender for Client {
    type Error = reqwest::Error;

    fn send<I, K, V, S>(&self, method_url: S, params: I) -> Result<String, Self::Error>
    where
        I: IntoIterator + Send,
        K: AsRef<str>,
        V: AsRef<str>,
        I::Item: Borrow<(K, V)>,
        S: AsRef<str> + Send,
    {
        let mut url = reqwest::Url::parse(method_url.as_ref()).expect("Unable to parse url");
        url.query_pairs_mut().extend_pairs(params);
        Ok(self.inner.get(url).send()?.text()?)
    }
}

pub fn build_client(d_cookie: Option<&str>) -> Result<Client, reqwest::Error> {
    // When sending requests using "xoxc" type token (lifted from a browser), it's also required
    // to send the "d" cookie together with it. Enhance the Slack API client to send this cookie
    // along the way.
    let mut headers = header::HeaderMap::new();
    if let Some(cookie_val) = d_cookie {
        let cookie = format!("d={}", cookie_val);
        headers.insert(
            "Cookie",
            header::HeaderValue::from_str(cookie.as_str()).expect("Cannot generate cookie"),
        );
    }
    let client = reqwest::blocking::ClientBuilder::new()
        .default_headers(headers)
        .build()?;
    Ok(Client { inner: client })
}
