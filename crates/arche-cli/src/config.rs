use crate::cli::GlobalOpts;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: String,
    pub api_key: Option<String>,
    pub verbose: bool,
    pub quiet: bool,
}

impl Config {
    pub fn from_global_opts(opts: &GlobalOpts) -> Self {
        Self {
            api_url: opts.api_url.clone(),
            api_key: opts.api_key.clone(),
            verbose: opts.verbose,
            quiet: opts.quiet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_opts(api_url: &str, api_key: Option<&str>, verbose: bool, quiet: bool) -> GlobalOpts {
        GlobalOpts {
            api_url: api_url.to_string(),
            api_key: api_key.map(|s| s.to_string()),
            verbose,
            quiet,
        }
    }

    #[test]
    fn test_config_from_opts_defaults() {
        let opts = make_opts("http://localhost:8080", None, false, false);
        let config = Config::from_global_opts(&opts);
        assert_eq!(config.api_url, "http://localhost:8080");
        assert_eq!(config.api_key, None);
        assert!(!config.verbose);
        assert!(!config.quiet);
    }

    #[test]
    fn test_config_from_opts_custom() {
        let opts = make_opts("http://example.com:9090", Some("sk-abc123"), true, true);
        let config = Config::from_global_opts(&opts);
        assert_eq!(config.api_url, "http://example.com:9090");
        assert_eq!(config.api_key, Some("sk-abc123".to_string()));
        assert!(config.verbose);
        assert!(config.quiet);
    }

    #[test]
    fn test_config_from_opts_api_key_none() {
        let opts = make_opts("http://localhost:8080", None, false, true);
        let config = Config::from_global_opts(&opts);
        assert_eq!(config.api_key, None);
        assert!(config.quiet);
    }
}
