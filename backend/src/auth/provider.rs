use hasync_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn generate_auth_url(&self) -> (String, String, String);

    async fn exchange_code(&self, code: String, verifier: String) -> Result<String, String>;

    async fn fetch_user_info(&self, access_token: &str) -> Result<Value, string>;
}
