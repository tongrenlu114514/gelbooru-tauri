use reqwest::{Client, cookie::Jar};
use std::sync::Arc;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct HttpClient {
    client: Client,
    jar: Arc<Jar>,
}

impl HttpClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let jar = Arc::new(Jar::default());
        let mut builder = Client::builder()
            .user_agent(USER_AGENT)
            .cookie_provider(jar.clone())
            .gzip(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(30));
        
        // 默认代理地址
        let proxy_url = "http://127.0.0.1:7897";
        if let Ok(proxy) = reqwest::Url::parse(proxy_url) {
            builder = builder.proxy(reqwest::Proxy::all(proxy)?);
            println!("[INFO] Using proxy: {}", proxy_url);
        }
        
        // 也可以通过环境变量覆盖
        if let Ok(proxy) = std::env::var("HTTP_PROXY")
            .or_else(|_| std::env::var("http_proxy"))
        {
            if let Ok(proxy_url) = reqwest::Url::parse(&proxy) {
                builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
                println!("[INFO] Using env proxy: {}", proxy);
            }
        }
        
        let client = builder.build()?;
        
        Ok(Self { client, jar })
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    pub async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        self.client
            .get(url)
            .send()
            .await?
            .text()
            .await
    }
    
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        self.client
            .get(url)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }
    
    pub async fn get_image_with_referer(&self, url: &str, referer: &str) -> Result<Vec<u8>, reqwest::Error> {
        self.client
            .get(url)
            .header("Referer", referer)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }
    
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
