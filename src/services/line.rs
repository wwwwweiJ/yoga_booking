//! LINE Login (LIFF) verification.
//!
//! A LIFF client hands us an `id_token` (a JWT LINE signed). Rather than verify
//! that signature ourselves, we hand it back to LINE's `verify` endpoint, which
//! checks the signature, audience and expiry server-side and returns the
//! trusted claims. That keeps the app free of any JWT crypto for the LINE path.
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use serde::Deserialize;

const VERIFY_URL: &str = "https://api.line.me/oauth2/v2.1/verify";

/// The trusted subset of a verified LINE ID token.
pub struct LineIdentity {
    /// LINE user id (`sub`) — stable per channel; our per-studio user key.
    pub sub: String,
    /// Display name (needs the `profile` scope); a fallback fills in if absent.
    pub name: String,
}

#[derive(Deserialize)]
struct VerifyOk {
    sub: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct VerifyErr {
    error_description: Option<String>,
}

/// Verify a LIFF `id_token` with LINE. `channel_id` is the LINE Login channel's
/// numeric Channel ID and must equal the token's `aud`.
///
/// # Errors
///
/// - `401` (Unauthorized) when LINE rejects the token (bad, expired, wrong aud).
/// - `500` when LINE is unreachable or replies with something unparseable — a
///   server-side fault, distinct from a bad token.
pub async fn verify_id_token(id_token: &str, channel_id: &str) -> Result<LineIdentity> {
    let res = reqwest::Client::new()
        .post(VERIFY_URL)
        .form(&[("id_token", id_token), ("client_id", channel_id)])
        .send()
        .await
        .map_err(|e| Error::string(&format!("could not reach LINE: {e}")))?;

    if !res.status().is_success() {
        let reason = res
            .json::<VerifyErr>()
            .await
            .ok()
            .and_then(|e| e.error_description)
            .unwrap_or_else(|| "invalid LINE token".to_string());
        return Err(Error::CustomError(
            StatusCode::UNAUTHORIZED,
            ErrorDetail::with_reason(reason),
        ));
    }

    let ok = res
        .json::<VerifyOk>()
        .await
        .map_err(|e| Error::string(&format!("unexpected LINE response: {e}")))?;

    Ok(LineIdentity {
        sub: ok.sub,
        name: ok.name.unwrap_or_else(|| "LINE 使用者".to_string()),
    })
}
