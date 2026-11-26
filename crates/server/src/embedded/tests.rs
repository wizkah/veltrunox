use super::*;
use reqwest::StatusCode;
use std::net::TcpListener;

fn start(directory: &std::path::Path) -> Embedded {
    Embedded::start(Places {
        data_dir: directory.to_path_buf(),
        web_dir: None,
    })
    .unwrap()
}

fn add_user(embedded: &Embedded) {
    let accounts = embedded.app.accounts.as_ref().unwrap();
    accounts.setup("admin", "secret-123").unwrap();
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("0.0.0.0:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn on(port: u16) -> SharingSettings {
    SharingSettings {
        enabled: true,
        port,
        https: false,
    }
}

/// The status of `GET /api/v1/me`; `None` if nothing answers.
fn me(base: &str, token: Option<&str>) -> Option<StatusCode> {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    runtime.block_on(async {
        let mut request = reqwest::Client::new().get(format!("{base}/api/v1/me"));
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        request.send().await.ok().map(|response| response.status())
    })
}

#[test]
fn the_owner_token_works_at_the_local_endpoint_only() {
    let directory = tempfile::tempdir().unwrap();
    let mut embedded = start(directory.path());
    let token = embedded.owner_token().to_owned();
    assert_eq!(me(embedded.local_url(), Some(&token)), Some(StatusCode::OK));
    assert_eq!(
        me(embedded.local_url(), None),
        Some(StatusCode::UNAUTHORIZED)
    );

    add_user(&embedded);
    let port = free_port();
    embedded.apply(on(port)).unwrap();
    let network = format!("http://127.0.0.1:{port}");
    assert_eq!(me(&network, Some(&token)), Some(StatusCode::UNAUTHORIZED));
}

#[test]
fn the_network_endpoint_needs_an_account() {
    let directory = tempfile::tempdir().unwrap();
    let mut embedded = start(directory.path());
    assert_eq!(
        embedded.apply(on(free_port())),
        Err(SharingError::NoAccount)
    );
    assert!(!embedded.status().settings.enabled);
}

#[test]
fn switching_off_closes_the_port_and_keeps_the_local_endpoint() {
    let directory = tempfile::tempdir().unwrap();
    let mut embedded = start(directory.path());
    add_user(&embedded);
    let port = free_port();
    let network = format!("http://127.0.0.1:{port}");
    embedded.apply(on(port)).unwrap();
    assert_eq!(me(&network, None), Some(StatusCode::UNAUTHORIZED));

    embedded
        .apply(SharingSettings {
            enabled: false,
            ..on(port)
        })
        .unwrap();
    // The runtime shuts down in the background.
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert_eq!(me(&network, None), None);
    let token = embedded.owner_token().to_owned();
    assert_eq!(me(embedded.local_url(), Some(&token)), Some(StatusCode::OK));
}

#[test]
fn a_busy_port_is_reported_and_not_remembered_as_on() {
    let directory = tempfile::tempdir().unwrap();
    let mut embedded = start(directory.path());
    add_user(&embedded);
    let busy = TcpListener::bind("0.0.0.0:0").unwrap();
    let port = busy.local_addr().unwrap().port();
    assert_eq!(embedded.apply(on(port)), Err(SharingError::PortInUse));
    assert_eq!(
        embedded.status().settings,
        SharingSettings {
            enabled: false,
            ..on(port)
        }
    );
}

#[test]
fn a_server_left_on_starts_with_the_app() {
    let directory = tempfile::tempdir().unwrap();
    let port = free_port();
    {
        let mut embedded = start(directory.path());
        add_user(&embedded);
        embedded.apply(on(port)).unwrap();
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    let embedded = start(directory.path());
    assert_eq!(embedded.status().failure, None);
    assert_eq!(
        me(&format!("http://127.0.0.1:{port}"), None),
        Some(StatusCode::UNAUTHORIZED)
    );
}
