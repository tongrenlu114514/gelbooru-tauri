use reqwest::{Client, cookie::Jar, Response};
use std::sync::Arc;
use tokio::sync::RwLock;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct HttpClient {
    client: RwLock<Client>,
    jar: Arc<Jar>,
}

impl HttpClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let jar = Arc::new(Jar::default());
        let default_proxy = Some("http://127.0.0.1:7897".to_string());
        
        let client = Self::build_client(&jar, default_proxy.as_deref())?;
        
        Ok(Self { 
            client: RwLock::new(client), 
            jar,
        })
    }
    
    fn build_client(jar: &Arc<Jar>, proxy_url: Option<&str>) -> Result<Client, Box<dyn std::error::Error>> {
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
    
    pub async fn set_proxy(&self, proxy_url: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let new_client = Self::build_client(&self.jar, proxy_url.as_deref())?;
        *self.client.write().await = new_client;
        println!("[INFO] Proxy updated: {:?}", proxy_url);
        Ok(())
    }
    
    pub async fn get(&self, url: &str) -> Result<String, reqwest::Error> {
        self.client.read().await
            .get(url)
            .send()
            .await?
            .text()
            .await
    }
    
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, reqwest::Error> {
        self.client.read().await
            .get(url)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }
    
    pub async fn get_image_with_referer(&self, url: &str, referer: &str) -> Result<Vec<u8>, reqwest::Error> {
        self.client.read().await
            .get(url)
            .header("Referer", referer)
            .send()
            .await?
            .bytes()
            .await
            .map(|b| b.to_vec())
    }
    
    pub async fn download_image(&self, url: &str, referer: &str) -> Result<Response, reqwest::Error> {
        self.client.read().await
            .get(url)
            .header("Referer", referer)
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
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
