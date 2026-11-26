//! The network endpoint while it is on. It has a runtime of its own, so that
//! switching it off also ends every open connection.

use super::{Places, SharingError, SharingSettings};
use crate::{
    config::{Config, TlsMode},
    serve::listen,
    tls, web,
};
use std::io;
use tokio::runtime::{Builder, Runtime};
use veltrunox_server_adapters::{AppState, router};

pub struct Sharing {
    runtime: Option<Runtime>,
    scheme: &'static str,
    port: u16,
}

impl Sharing {
    pub fn open(
        app: &AppState,
        places: &Places,
        wanted: SharingSettings,
    ) -> Result<Self, SharingError> {
        let failed = |error: &dyn std::fmt::Display| SharingError::Failed(error.to_string());
        let config = Config {
            data_dir: places.data_dir.clone(),
            tls: if wanted.https {
                TlsMode::LocalCa
            } else {
                TlsMode::Off
            },
            ..Config::default()
        };
        let tls = tls::prepare(&config).map_err(|error| failed(&error))?;
        let routes = web::with_web_app(router(app.clone()), places.web_dir.as_deref());
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|error| failed(&error))?;
        let (address, serving) = listen(&format!("0.0.0.0:{}", wanted.port), routes, tls).map_err(
            |error| match error.kind() {
                io::ErrorKind::AddrInUse => SharingError::PortInUse,
                _ => failed(&error),
            },
        )?;
        runtime.spawn(async move {
            if let Err(error) = serving.await {
                eprintln!("the network endpoint stopped: {error}");
            }
        });
        Ok(Self {
            runtime: Some(runtime),
            scheme: if wanted.https { "https" } else { "http" },
            port: address.port(),
        })
    }

    /// One address per way to reach this machine, the local name first.
    pub fn urls(&self) -> Vec<String> {
        tls::reachable_hosts()
            .into_iter()
            .map(|host| format!("{}://{host}:{}", self.scheme, self.port))
            .collect()
    }
}

impl Drop for Sharing {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
