//! `veltrunox-mcp`: serves the bookmark tools to an MCP client over standard input and output.
//!
//! The server is named by `--url <address>` or `VELTRUNOX_URL` (default `http://127.0.0.1:8080`);
//! the API token comes from `VELTRUNOX_TOKEN` only, so that it does not show up in a process list.
//! For a server with its own certificate authority, `VELTRUNOX_CA` names that certificate file.

use std::process::ExitCode;
use veltrunox_mcp::{HttpBackend, serve};

const DEFAULT_URL: &str = "http://127.0.0.1:8080";

fn url_from(args: &[String], environment: Option<String>) -> String {
    let from_option = args
        .iter()
        .position(|argument| argument == "--url")
        .and_then(|index| args.get(index + 1).cloned());
    from_option
        .or(environment)
        .unwrap_or_else(|| DEFAULT_URL.to_owned())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Ok(token) = std::env::var("VELTRUNOX_TOKEN") else {
        eprintln!("veltrunox-mcp: set VELTRUNOX_TOKEN to an API token of the server");
        return ExitCode::from(2);
    };
    let url = url_from(&args, std::env::var("VELTRUNOX_URL").ok());
    let authority = match std::env::var_os("VELTRUNOX_CA").map(std::fs::read) {
        None => None,
        Some(Ok(pem)) => Some(pem),
        Some(Err(error)) => {
            eprintln!("veltrunox-mcp: cannot read VELTRUNOX_CA: {error}");
            return ExitCode::from(2);
        }
    };
    let backend = match HttpBackend::new(&url, &token, authority.as_deref()) {
        Ok(backend) => backend,
        Err(error) => {
            eprintln!("veltrunox-mcp: {error}");
            return ExitCode::from(1);
        }
    };
    match serve(&backend, std::io::stdin().lock(), std::io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("veltrunox-mcp: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|text| (*text).to_owned()).collect()
    }

    #[test]
    fn the_option_beats_the_environment_beats_the_default() {
        let env = Some("http://env:1".to_owned());
        assert_eq!(
            url_from(&args(&["--url", "http://opt:2"]), env.clone()),
            "http://opt:2"
        );
        assert_eq!(url_from(&args(&[]), env), "http://env:1");
        assert_eq!(url_from(&args(&[]), None), DEFAULT_URL);
        assert_eq!(url_from(&args(&["--url"]), None), DEFAULT_URL);
    }
}
