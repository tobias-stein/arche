use crate::config::Config;
use reqwest::Client as HttpClient;

pub fn build_client(config: &Config) -> HttpClient {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(api_key) = &config.api_key {
        let mut value =
            reqwest::header::HeaderValue::from_str(api_key).expect("Invalid API key header value");
        value.set_sensitive(true);
        headers.insert("X-API-Key", value);
    }
    HttpClient::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(api_url: &str, api_key: Option<&str>) -> Config {
        Config {
            api_url: api_url.to_string(),
            api_key: api_key.map(|s| s.to_string()),
            verbose: false,
            quiet: false,
        }
    }

    #[test]
    fn test_build_client_succeeds() {
        let config = make_config("http://localhost:8080", Some("sk-test"));
        let _client = build_client(&config);
    }

    #[test]
    fn test_build_client_no_api_key() {
        let config = make_config("http://localhost:8080", None);
        let _client = build_client(&config);
    }
}
