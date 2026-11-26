//! A browser has no server inside.

use crate::host_api::{LocalHost, SharingChoice, SharingInfo, SharingProblem};

const NONE: &str = "a browser carries no server";

pub fn start() -> Result<LocalHost, String> {
    Err(NONE.to_owned())
}

pub fn info() -> Result<SharingInfo, SharingProblem> {
    Err(SharingProblem::Failed {
        detail: NONE.to_owned(),
    })
}

pub fn apply(_choice: SharingChoice) -> Result<SharingInfo, SharingProblem> {
    info()
}
