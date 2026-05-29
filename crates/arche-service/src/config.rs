use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub cache_poll_interval_ms: u64,
    pub db_pool_size: u32,
}

impl Config {
    pub fn from_env() -> Self {
        let default_pool_size = (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) * 2 + 10) as u32;
        Self {
            port: env::var("ARCHE_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("ARCHE_DATABASE_URL").expect("ARCHE_DATABASE_URL must be set"),
            redis_url: env::var("ARCHE_REDIS_URL").ok(),
            cache_poll_interval_ms: env::var("ARCHE_CACHE_POLL_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60_000),
            db_pool_size: env::var("ARCHE_DB_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default_pool_size),
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}
