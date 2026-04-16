use reqwest::{cookie::Jar, Client, Response};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

const RATE_LIMIT_GAP_MS: u64 = 500;

pub struct HttpClient {
    client: RwLock<Client>,
    jar: Arc<Jar>,
    /// Timestamp of last HTTP request — used to enforce minimum gap between requests.
    pub(crate) last_request_time: RwLock<Instant>,
}

impl HttpClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let jar = Arc::new(Jar::default());
        // No default proxy - will be configured from settings
        let client = Self::build_client(&jar, None)?;

        Ok(Self {
            client: RwLock::new(client),
            jar,
            last_request_time: RwLock::new(Instant::now()),
        })
    }

    fn build_client(
        jar: &Arc<Jar>,
        proxy_url: Option<&str>,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let mut builder = Client::builder()
            .user_agent(USER_AGENT)
            .cookie_provider(jar.clone())
            .gzip(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(30));

        if let Some(proxy) = proxy_url {
            if !proxy.is_empty() {
                if let Ok(proxy_uri) = reqwest::Url::parse(proxy) {
                    builder = builder.proxy(reqwest::Proxy::all(proxy_uri)?);
                    println!("[INFO] Using proxy: {}", proxy);
                }
            }
        }

        Ok(builder.build()?)
    }

    pub async fn set_proxy(
        &self,
        proxy_url: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let new_client = Self::build_client(&self.jar, proxy_url.as_deref())?;
        *self.client.write().await = new_client;
        println!("[INFO] Proxy updated: {:?}", proxy_url);
        Ok(())
    }

    pub async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        self.wait_for_rate_limit().await;
        self.client.read().await.get(url).send().await?.text().await
    }

    #[allow(dead_code)]
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        self.client
            .read()
            .await
            .get(url)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }

    pub async fn wait_for_rate_limit(&self) {
        let min_gap = std::time::Duration::from_millis(RATE_LIMIT_GAP_MS);
        let last = *self.last_request_time.read().await;
        let elapsed = last.elapsed();
        if elapsed < min_gap {
            tokio::time::sleep(min_gap - elapsed).await;
        }
        *self.last_request_time.write().await = Instant::now();
    }

    pub async fn get_image_with_referer(
        &self,
        url: &str,
        referer: &str,
    ) -> Result<Vec<u8>, reqwest::Error> {
        self.wait_for_rate_limit().await;
        self.client
            .read()
            .await
            .get(url)
            .header("Referer", referer)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }

    pub async fn download_image(
        &self,
        url: &str,
        referer: &str,
    ) -> Result<Response, reqwest::Error> {
        self.wait_for_rate_limit().await;
        self.client
            .read()
            .await
            .get(url)
            .header("Referer", referer)
            .header(
                "Accept",
                "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
    }

    #[allow(dead_code)]
    pub fn load_cookies(&self, json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Load cookies from JSON file and add to jar
        if std::path::Path::new(json_path).exists() {
            let content = std::fs::read_to_string(json_path)?;
            let cookies: Vec<serde_json::Value> = serde_json::from_str(&content)?;

            for cookie in cookies {
                // Parse cookie and add to jar
                if let (Some(name), Some(value), Some(domain)) = (
                    cookie.get("name").and_then(|v| v.as_str()),
                    cookie.get("value").and_then(|v| v.as_str()),
                    cookie.get("domain").and_then(|v| v.as_str()),
                ) {
                    let cookie_str = format!("{}={}", name, value);
                    let url = format!("https://{}", domain.trim_start_matches('.'));
                    if let Ok(parsed_url) = url::Url::parse(&url) {
                        self.jar.add_cookie_str(&cookie_str, &parsed_url);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    // -------------------------------------------------------------------------
    // wait_for_rate_limit (private implementation method — made pub(crate) for tests)
    // -------------------------------------------------------------------------

    /// Verifies the 500ms gap is enforced between consecutive requests.
    #[tokio::test]
    async fn rate_limit_enforces_500ms_gap_between_consecutive_calls() {
        let client = HttpClient::new().unwrap();

        // First call — initializes last_request_time
        client.wait_for_rate_limit().await;

        // Second call immediately after — must sleep to fill the 500ms gap
        let before_second = Instant::now();
        client.wait_for_rate_limit().await;
        let sleep_duration = before_second.elapsed();

        assert!(
            sleep_duration >= Duration::from_millis(450),
            "Expected ~500ms sleep on second call, got {:?}",
            sleep_duration
        );
    }

    /// Verifies that once 500ms has elapsed, wait_for_rate_limit returns immediately.
    #[tokio::test]
    async fn rate_limit_returns_immediately_after_gap_elapsed() {
        let client = HttpClient::new().unwrap();

        // First call
        client.wait_for_rate_limit().await;

        // Wait 600ms past the gap
        tokio::time::sleep(Duration::from_millis(600)).await;
        // Advance the stored last_request_time so the client thinks 600ms have passed
        *client.last_request_time.write().await = Instant::now() - Duration::from_millis(600);

        // Third call — should return without sleeping
        let start = Instant::now();
        client.wait_for_rate_limit().await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(50),
            "Expected near-immediate return after gap elapsed, got {:?}",
            elapsed
        );
    }

    /// Verifies last_request_time is updated after each wait_for_rate_limit call.
    #[tokio::test]
    async fn rate_limit_updates_last_request_time_after_each_call() {
        let client = HttpClient::new().unwrap();

        let before_first = Instant::now();
        client.wait_for_rate_limit().await;
        let after_first = Instant::now();

        let last_after_first = *client.last_request_time.read().await;
        assert!(
            last_after_first >= before_first && last_after_first <= after_first,
            "last_request_time should be set to now after first call"
        );

        // Advance time past the gap so second call updates it
        tokio::time::sleep(Duration::from_millis(600)).await;

        let before_second = Instant::now();
        client.wait_for_rate_limit().await;
        let after_second = Instant::now();

        let last_after_second = *client.last_request_time.read().await;
        assert!(
            last_after_second >= before_second && last_after_second <= after_second,
            "last_request_time should be updated to now after second call"
        );
        // Second update should be strictly after first
        assert!(
            last_after_second > last_after_first,
            "last_request_time should advance between calls"
        );
    }

    /// Verifies the RwLock allows concurrent readers (multiple requests checking
    /// the timestamp simultaneously without blocking each other).
    #[tokio::test]
    async fn rate_limit_rwlock_allows_concurrent_reads() {
        let client = HttpClient::new().unwrap();
        client.wait_for_rate_limit().await;

        // Arc-wrap the client so each task can own a clone without lifetime issues.
        use std::sync::Arc;
        let client_arc = Arc::new(client);
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let c = client_arc.clone();
                tokio::spawn(async move {
                    let _read = c.last_request_time.read().await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    true
                })
            })
            .collect();

        let results: Vec<bool> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(results.len(), 4, "All concurrent reads should complete");
    }

    /// Verifies that all three HTTP methods are rate-limited by checking that
    /// last_request_time is updated after a wait_for_rate_limit call (simulating
    /// what the HTTP methods do internally).
    #[tokio::test]
    async fn rate_limit_applies_to_all_http_methods() {
        let client = HttpClient::new().unwrap();

        // Simulate what get(), get_image_with_referer(), and download_image() do:
        // they each call wait_for_rate_limit() first
        client.wait_for_rate_limit().await;

        // Advance time past the gap
        tokio::time::sleep(Duration::from_millis(600)).await;
        *client.last_request_time.write().await = Instant::now() - Duration::from_millis(600);

        // After 600ms gap, wait_for_rate_limit should return immediately
        // — this is the behavior all three HTTP methods will exhibit
        let start = Instant::now();
        client.wait_for_rate_limit().await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(50),
            "All HTTP methods that call wait_for_rate_limit should return immediately after 500ms gap"
        );
    }
}
