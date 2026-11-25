//! The Veltrunox server as the tools see it: two kinds of request and an answer.

use serde_json::Value;
use std::time::Duration;

/// What the server answered: the status and the body as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub status: u16,
    pub body: String,
}

impl Reply {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// The server could not be reached or its answer could not be read.
#[derive(Debug, thiserror::Error)]
#[error("cannot reach the Veltrunox server: {0}")]
pub struct BackendError(pub String);

/// Port: the HTTP API of the server.
pub trait Backend {
    fn get(&self, path: &str) -> Result<Reply, BackendError>;
    fn post_json(&self, path: &str, body: &Value) -> Result<Reply, BackendError>;
}

const TIMEOUT: Duration = Duration::from_secs(30);

/// Calls the server over HTTP with an API token.
pub struct HttpBackend {
    base_url: String,
    token: String,
    client: reqwest::blocking::Client,
}

impl HttpBackend {
    pub fn new(base_url: &str, token: &str) -> Result<Self, BackendError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(|error| BackendError(error.to_string()))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            token: token.to_owned(),
            client,
        })
    }

    fn send(&self, request: reqwest::blocking::RequestBuilder) -> Result<Reply, BackendError> {
        let response = request
            .bearer_auth(&self.token)
            .send()
            .map_err(|error| BackendError(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|error| BackendError(error.to_string()))?;
        Ok(Reply { status, body })
    }
}

impl Backend for HttpBackend {
    fn get(&self, path: &str) -> Result<Reply, BackendError> {
        self.send(self.client.get(format!("{}{path}", self.base_url)))
    }

    fn post_json(&self, path: &str, body: &Value) -> Result<Reply, BackendError> {
        self.send(
            self.client
                .post(format!("{}{path}", self.base_url))
                .json(body),
        )
    }
}
