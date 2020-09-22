//! The HTTP client may vary depending on which one the user configures. This
//! module contains the required logic to use different clients
//! interchangeably.

#[cfg(feature = "client-reqwest")]
mod reqwest;
#[cfg(feature = "client-ureq")]
mod ureq;

use crate::client::ClientResult;

use std::collections::HashMap;

use maybe_async::maybe_async;
use serde_json::Value;

pub type Headers = HashMap<String, String>;
pub type FormData = HashMap<String, String>;

pub mod headers {
    pub const AUTHORIZATION: &str = "authorization";

    /// Generates an HTTP token authorization header with proper formatting
    pub fn bearer_auth(token: &str) -> String {
        format!("Bearer {}", token)
    }

    /// Generates an HTTP basic authorization header with proper formatting
    pub fn basic_auth(user: &str, password: &str) -> String {
        let value = format!("{}:{}", user, password);
        format!("Basic {}", base64::encode(value))
    }
}

/// The default headers will be overriden if its value is other than None.
#[maybe_async]
pub trait BaseClient {
    async fn get(
        &self,
        url: &str,
        headers: Option<&Headers>,
        params: &Value,
    ) -> ClientResult<String>;

    async fn post(
        &self,
        url: &str,
        headers: Option<&Headers>,
        payload: &Value,
    ) -> ClientResult<String>;

    async fn post_form(
        &self,
        url: &str,
        headers: Option<&Headers>,
        payload: &FormData,
    ) -> ClientResult<String>;

    async fn put(
        &self,
        url: &str,
        headers: Option<&Headers>,
        payload: &Value,
    ) -> ClientResult<String>;

    async fn delete(
        &self,
        url: &str,
        headers: Option<&Headers>,
        payload: &Value,
    ) -> ClientResult<String>;
}