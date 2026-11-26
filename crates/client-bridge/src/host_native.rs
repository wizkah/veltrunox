//! The server inside the desktop app.

use crate::host_api::{LocalHost, SharingChoice, SharingInfo, SharingProblem};
use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, PoisonError},
};
use veltrunox_server::embedded::{Embedded, Places, SharingError, SharingSettings, SharingStatus};

static HOST: Mutex<Option<Embedded>> = Mutex::new(None);

fn running() -> MutexGuard<'static, Option<Embedded>> {
    HOST.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn start() -> Result<LocalHost, String> {
    let mut host = running();
    if host.is_none() {
        *host = Some(Embedded::start(places()?).map_err(|error| error.to_string())?);
    }
    host.as_ref()
        .map(|host| LocalHost {
            url: host.local_url().to_owned(),
            owner_token: host.owner_token().to_owned(),
        })
        .ok_or_else(|| "the server did not start".to_owned())
}

pub fn info() -> Result<SharingInfo, SharingProblem> {
    running()
        .as_ref()
        .map(|host| info_of(host.status()))
        .ok_or_else(not_started)
}

pub fn apply(choice: SharingChoice) -> Result<SharingInfo, SharingProblem> {
    let mut host = running();
    let host = host.as_mut().ok_or_else(not_started)?;
    host.apply(SharingSettings {
        enabled: choice.enabled,
        port: choice.port,
        https: choice.https,
    })
    .map(info_of)
    .map_err(problem_of)
}

/// The data of the app, and the web app inside its bundle. Both can be
/// redirected for development and tests.
fn places() -> Result<Places, String> {
    let data_dir = match std::env::var_os("VELTRUNOX_DATA_DIR") {
        Some(directory) => PathBuf::from(directory),
        None => std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/Veltrunox"))
            .ok_or("no home directory")?,
    };
    Ok(Places {
        data_dir,
        web_dir: web_directory(),
    })
}

fn web_directory() -> Option<PathBuf> {
    let directory = std::env::var_os("VELTRUNOX_WEB_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            // <bundle>/Contents/MacOS/<program>
            let program = std::env::current_exe().ok()?;
            Some(program.parent()?.parent()?.join("Resources/web"))
        })?;
    directory.join("index.html").is_file().then_some(directory)
}

fn not_started() -> SharingProblem {
    SharingProblem::Failed {
        detail: "the server is not running".to_owned(),
    }
}

fn info_of(status: SharingStatus) -> SharingInfo {
    SharingInfo {
        choice: SharingChoice {
            enabled: status.settings.enabled,
            port: status.settings.port,
            https: status.settings.https,
        },
        urls: status.urls,
        problem: status.failure.map(problem_of),
        has_web_app: status.has_web_app,
    }
}

fn problem_of(error: SharingError) -> SharingProblem {
    match error {
        SharingError::NoAccess => SharingProblem::NoAccess,
        SharingError::PortInUse => SharingProblem::PortInUse,
        SharingError::Failed(detail) => SharingProblem::Failed { detail },
    }
}
