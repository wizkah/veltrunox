//! Transport security (ADR-008): none, a certificate authority of the server's
//! own, or certificate files kept by the operator.

mod local_ca;

use crate::config::{Config, TlsMode};
use axum::{Router, http::header, routing::get};
use axum_server::tls_rustls::RustlsConfig;
use local_ca::{CA_CERT_FILE, Issued, LocalCa, ServerNames};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use time::OffsetDateTime;

/// Where the authority certificate can be downloaded for installing it.
pub const CA_DOWNLOAD_PATH: &str = "/ca.crt";
const TLS_DIRECTORY: &str = "tls";
const RENEWAL_CHECK: Duration = Duration::from_secs(60);

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("cannot access {}: {}", .0.display(), .1)]
    File(PathBuf, std::io::Error),
    #[error("{} holds only half of the certificate authority; restore it or delete both ca files", .0.display())]
    IncompleteAuthority(PathBuf),
    #[error("tls = \"files\" needs tls_cert_file and tls_key_file")]
    MissingFiles,
    #[error("cannot create a certificate: {0}")]
    Generate(#[from] rcgen::Error),
    #[error("invalid certificate or key: {0}")]
    Pem(#[from] rustls::pki_types::pem::Error),
    #[error("unusable certificate or key: {0}")]
    Rustls(#[from] rustls::Error),
}

/// What the HTTPS endpoint needs.
pub struct Tls {
    pub config: RustlsConfig,
    /// Set in `local-ca` mode.
    authority: Option<Authority>,
}

struct Authority {
    ca: LocalCa,
    certificate_pem: String,
    served: Issued,
}

/// Prepares the certificate the configuration asks for; `None` for plain HTTP.
pub fn prepare(config: &Config) -> Result<Option<Tls>, TlsError> {
    match config.tls {
        TlsMode::Off => Ok(None),
        TlsMode::Files => from_files(config).map(Some),
        TlsMode::LocalCa => from_local_ca(config).map(Some),
    }
}

fn from_files(config: &Config) -> Result<Tls, TlsError> {
    let (Some(cert), Some(key)) = (&config.tls_cert_file, &config.tls_key_file) else {
        return Err(TlsError::MissingFiles);
    };
    let read = |path: &PathBuf| {
        std::fs::read_to_string(path).map_err(|error| TlsError::File(path.clone(), error))
    };
    let server = server_config(&Issued {
        chain_pem: read(cert)?,
        key_pem: read(key)?,
    })?;
    Ok(Tls {
        config: RustlsConfig::from_config(server),
        authority: None,
    })
}

fn from_local_ca(config: &Config) -> Result<Tls, TlsError> {
    let directory = config.data_dir.join(TLS_DIRECTORY);
    let now = OffsetDateTime::now_utc();
    let ca = LocalCa::open(&directory, &hostname(), now)?;
    let names = current_names();
    let served = ca.server_certificate(&names, now)?;
    let certificate_pem = ca.certificate_pem()?;
    println!(
        "certificate for {}; install {} on each device (SHA-256 {})",
        names.all().join(", "),
        directory.join(CA_CERT_FILE).display(),
        fingerprint(&certificate_pem)?
    );
    Ok(Tls {
        config: RustlsConfig::from_config(server_config(&served)?),
        authority: Some(Authority {
            ca,
            certificate_pem,
            served,
        }),
    })
}

impl Tls {
    /// Offers the authority certificate for download, if there is one.
    pub fn with_ca_download(&self, router: Router) -> Router {
        let Some(authority) = &self.authority else {
            return router;
        };
        let pem = authority.certificate_pem.clone();
        router.route(
            CA_DOWNLOAD_PATH,
            get(move || async move {
                (
                    [
                        (header::CONTENT_TYPE, "application/x-x509-ca-cert"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"veltrunox-ca.crt\"",
                        ),
                    ],
                    pem,
                )
            }),
        )
    }

    /// Keeps the served certificate valid for the current addresses and ahead
    /// of its expiry. Returns at once unless the authority is the server's own.
    pub fn keep_current(self) {
        let Some(mut authority) = self.authority else {
            return;
        };
        let config = self.config;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(RENEWAL_CHECK).await;
                if let Err(error) = renew(&mut authority, &config) {
                    eprintln!("cannot renew the certificate: {error}");
                }
            }
        });
    }
}

fn renew(authority: &mut Authority, config: &RustlsConfig) -> Result<(), TlsError> {
    let names = current_names();
    let issued = authority
        .ca
        .server_certificate(&names, OffsetDateTime::now_utc())?;
    if issued != authority.served {
        config.reload_from_config(server_config(&issued)?);
        authority.served = issued;
        println!("certificate renewed for {}", names.all().join(", "));
    }
    Ok(())
}

fn hostname() -> String {
    gethostname::gethostname().to_string_lossy().into_owned()
}

fn current_names() -> ServerNames {
    let addresses = if_addrs::get_if_addrs().unwrap_or_else(|error| {
        eprintln!("cannot list network addresses: {error}");
        Vec::new()
    });
    ServerNames::new(&hostname(), addresses.iter().map(|found| found.ip()))
}

fn server_config(issued: &Issued) -> Result<Arc<ServerConfig>, TlsError> {
    let chain = CertificateDer::pem_slice_iter(issued.chain_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_slice(issued.key_pem.as_bytes())?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut server = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_no_client_auth()
        .with_single_cert(chain, key)?;
    // The real-time channel upgrades an HTTP/1.1 request to a WebSocket.
    server.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(server))
}

/// The SHA-256 fingerprint as devices show it, to compare before trusting.
fn fingerprint(certificate_pem: &str) -> Result<String, TlsError> {
    let der = CertificateDer::from_pem_slice(certificate_pem.as_bytes())?;
    let digest = ring::digest::digest(&ring::digest::SHA256, &der);
    let bytes: Vec<String> = digest.as_ref().iter().map(|b| format!("{b:02X}")).collect();
    Ok(bytes.join(":"))
}
