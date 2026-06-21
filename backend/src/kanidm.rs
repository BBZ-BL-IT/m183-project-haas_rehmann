//! Minimal Kanidm REST client used for self-registration.
//!
//! On `POST /register` the backend creates a Kanidm person, adds it to the
//! casino users group, and mints a credential-reset token so the new user can
//! set a password. It authenticates with a Kanidm **service-account API token**
//! (provisioned with people + group admin rights – see devops/provision.sh).
//!
//! Registration is optional: if no token is configured, `from_env` returns
//! `None` and the `/register` endpoint reports it as unavailable.

use std::{env, fs, time::Duration};

use reqwest::{Certificate, Client, StatusCode};
use serde_json::json;

use crate::error::AppError;

#[derive(Clone)]
pub struct KanidmRegistrar {
    client: Client,
    /// Internal API base, e.g. `https://kanidm:8443`.
    api_url: String,
    /// Browser-facing origin for the reset link, e.g. `https://localhost:8443`.
    public_origin: String,
    token: String,
    group: String,
    reset_ttl: i64,
}

impl KanidmRegistrar {
    pub fn from_env() -> Option<Self> {
        let token = env_or_file("KANIDM_REGISTRAR_TOKEN")?;
        let api_url = env::var("KANIDM_API_URL")
            .ok()?
            .trim_end_matches('/')
            .to_string();
        let public_origin = env::var("KANIDM_PUBLIC_ORIGIN")
            .unwrap_or_else(|_| api_url.clone())
            .trim_end_matches('/')
            .to_string();
        let group = env::var("KANIDM_REGISTER_GROUP").unwrap_or_else(|_| "casino_users".to_string());

        let mut builder = Client::builder().timeout(Duration::from_secs(10));
        if let Ok(ca_path) = env::var("OIDC_CA_CERT") {
            match fs::read(&ca_path).map(|pem| Certificate::from_pem(&pem)) {
                Ok(Ok(cert)) => builder = builder.add_root_certificate(cert),
                _ => tracing::warn!("registrar: could not load OIDC_CA_CERT {ca_path}"),
            }
        }
        let client = builder.build().ok()?;

        tracing::info!("user self-registration enabled (group: {group})");
        Some(Self {
            client,
            api_url,
            public_origin,
            token,
            group,
            reset_ttl: 86_400,
        })
    }

    /// Create the person, add it to the group, and return a credential-reset URL.
    pub async fn register(&self, username: &str) -> Result<String, AppError> {
        self.create_person(username).await?;
        self.add_to_group(username).await?;
        let token = self.reset_token(username).await?;
        Ok(format!("{}/ui/reset?token={}", self.public_origin, token))
    }

    async fn create_person(&self, username: &str) -> Result<(), AppError> {
        let res = self
            .client
            .post(format!("{}/v1/person", self.api_url))
            .bearer_auth(&self.token)
            .json(&json!({ "attrs": { "name": [username], "displayname": [username] } }))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("kanidm create person: {e}")))?;

        match res.status() {
            s if s.is_success() => Ok(()),
            StatusCode::BAD_REQUEST | StatusCode::CONFLICT => Err(AppError::BadRequest(
                "username is already taken or not allowed".to_string(),
            )),
            s => Err(AppError::Internal(format!(
                "kanidm create person failed: {s}"
            ))),
        }
    }

    async fn add_to_group(&self, username: &str) -> Result<(), AppError> {
        let res = self
            .client
            .post(format!("{}/v1/group/{}/_attr/member", self.api_url, self.group))
            .bearer_auth(&self.token)
            .json(&json!([username]))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("kanidm group add: {e}")))?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Internal(format!(
                "kanidm group add failed: {}",
                res.status()
            )))
        }
    }

    async fn reset_token(&self, username: &str) -> Result<String, AppError> {
        let res = self
            .client
            .get(format!(
                "{}/v1/person/{}/_credential/_update_intent/{}",
                self.api_url, username, self.reset_ttl
            ))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("kanidm reset intent: {e}")))?;
        if !res.status().is_success() {
            return Err(AppError::Internal(format!(
                "kanidm reset intent failed: {}",
                res.status()
            )));
        }
        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("kanidm reset intent decode: {e}")))?;
        body.get("token")
            .and_then(|t| t.as_str())
            .map(str::to_string)
            .ok_or_else(|| AppError::Internal("kanidm reset intent missing token".to_string()))
    }
}

/// Read `NAME` from env, falling back to the file at `NAME_FILE`.
fn env_or_file(name: &str) -> Option<String> {
    if let Ok(v) = env::var(name) {
        if !v.is_empty() {
            return Some(v);
        }
    }
    if let Ok(path) = env::var(format!("{name}_FILE")) {
        return fs::read_to_string(path).ok().map(|s| s.trim().to_string());
    }
    None
}
