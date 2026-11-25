//! A Model Context Protocol server for working out bookmarks with an LLM. It speaks MCP over
//! standard input and output and reaches the Veltrunox server through its HTTP API with an
//! API token, so it can do no more than that token may.

mod backend;
mod server;
#[cfg(test)]
mod testing;
mod tools;

pub use backend::{Backend, BackendError, HttpBackend, Reply};
pub use server::{handle_message, serve};
