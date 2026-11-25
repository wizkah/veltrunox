//! A server that records what it was asked and answers as told.

use crate::backend::{Backend, BackendError, Reply};
use serde_json::Value;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Get(String),
    Post(String, Value),
}

pub struct FakeBackend {
    pub requests: RefCell<Vec<Request>>,
    answer: Result<Reply, String>,
}

impl FakeBackend {
    pub fn answering(status: u16, body: &str) -> Self {
        Self {
            requests: RefCell::default(),
            answer: Ok(Reply {
                status,
                body: body.to_owned(),
            }),
        }
    }

    pub fn unreachable() -> Self {
        Self {
            requests: RefCell::default(),
            answer: Err("connection refused".to_owned()),
        }
    }

    fn reply(&self, request: Request) -> Result<Reply, BackendError> {
        self.requests.borrow_mut().push(request);
        self.answer.clone().map_err(BackendError)
    }
}

impl Backend for FakeBackend {
    fn get(&self, path: &str) -> Result<Reply, BackendError> {
        self.reply(Request::Get(path.to_owned()))
    }

    fn post_json(&self, path: &str, body: &Value) -> Result<Reply, BackendError> {
        self.reply(Request::Post(path.to_owned(), body.clone()))
    }
}
