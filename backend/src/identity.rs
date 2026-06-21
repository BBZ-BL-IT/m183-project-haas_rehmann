//! Resolve the authenticated identity (and its roles) from the OIDC session.
//!
//! The `axum-oidc-client` `AuthSession` extractor already guarantees the caller
//! holds a valid, non-expired session (it returns `401` otherwise). Here we only
//! decode the tokens to read the identity *claims* — the signature was verified
//! during the code exchange, so an unverified decode of our own session token is
//! safe.
//!
//! ## Roles / admin detection
//!
//! Identity providers expose group membership differently (Kanidm uses group
//! SPNs like `casino_admins@localhost`, Keycloak nests them under
//! `realm_access.roles`, …). Rather than hard-code one shape, we gather every
//! role/group/scope string we can find from both the ID and access token and
//! flag the user as an admin when any of them contains the configured marker
//! (default substring `"admin"`). Configure via env:
//!
//! - `OIDC_ROLE_CLAIMS`     – comma separated claim names to scan
//!                            (default: `roles,groups,scopes,scope,entitlements`)
//! - `OIDC_ADMIN_ROLE`      – substring that marks an admin (default: `admin`)

use std::env;

use axum_oidc_client::auth_session::AuthSession;
use axum_oidc_client::jwt::{OidcClaims, decode_jwt_unverified};
use serde_json::Value;

use crate::error::AppError;

/// The resolved caller identity used by the handlers.
pub struct Identity {
    /// OIDC subject – the stable per-user key we store in the database.
    pub subject: String,
    /// Human friendly display name (preferred_username → name → email → sub).
    pub appname: String,
    pub email: Option<String>,
    pub is_admin: bool,
    /// Roles exposed to the frontend: `["user"]` or `["user", "admin"]`.
    pub roles: Vec<String>,
}

impl Identity {
    pub fn from_session(session: &AuthSession) -> Result<Self, AppError> {
        // The ID token carries the user's identity claims.
        let (_, id_claims) = decode_jwt_unverified(&session.id_token)
            .map_err(|e| AppError::Unauthorized(format!("invalid id token: {e}")))?;

        // The access token (also a JWT for Kanidm) often carries the richer
        // scope/role information, so scan it too when it decodes.
        let access_claims = decode_jwt_unverified(&session.access_token)
            .ok()
            .map(|(_, c)| c);

        let subject = id_claims.sub.clone();
        let appname = display_name(&id_claims, &subject);
        let email = id_claims.email.clone();

        // Collect role-ish strings from both tokens.
        let mut roles_seen: Vec<String> = Vec::new();
        collect_role_strings(&id_claims, &mut roles_seen);
        if let Some(ac) = &access_claims {
            collect_role_strings(ac, &mut roles_seen);
        }

        let admin_marker = env::var("OIDC_ADMIN_ROLE")
            .unwrap_or_else(|_| "admin".to_string())
            .to_lowercase();
        let is_admin = roles_seen
            .iter()
            .any(|r| r.to_lowercase().contains(&admin_marker));

        let mut roles = vec!["user".to_string()];
        if is_admin {
            roles.push("admin".to_string());
        }

        Ok(Identity {
            subject,
            appname,
            email,
            is_admin,
            roles,
        })
    }

    /// Guard for admin-only handlers.
    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.is_admin {
            Ok(())
        } else {
            Err(AppError::Forbidden("admin role required".to_string()))
        }
    }
}

/// Best display name from the standard OIDC profile claims.
fn display_name(claims: &OidcClaims, subject: &str) -> String {
    claims
        .extra
        .get("preferred_username")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| claims.name.clone())
        .or_else(|| claims.email.clone())
        .unwrap_or_else(|| subject.to_owned())
}

/// Pull every string out of the configured role/group/scope claims, splitting
/// space- and comma-separated scope strings into individual entries.
fn collect_role_strings(claims: &OidcClaims, out: &mut Vec<String>) {
    let claim_names = env::var("OIDC_ROLE_CLAIMS")
        .unwrap_or_else(|_| "roles,groups,scopes,scope,entitlements".to_string());

    for name in claim_names.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(value) = claims.extra.get(name) {
            push_value(value, out);
        }
    }
}

fn push_value(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            // A single "scope" claim is space/comma separated.
            for part in s.split([' ', ',']).filter(|p| !p.is_empty()) {
                out.push(part.to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                push_value(item, out);
            }
        }
        // Some providers nest roles, e.g. {"realm_access": {"roles": [...]}}.
        Value::Object(map) => {
            for nested in map.values() {
                push_value(nested, out);
            }
        }
        _ => {}
    }
}
