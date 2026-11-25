//! The server's own certificate authority and the server certificate it issues,
//! both kept in the data directory so that devices keep trusting the server.

use super::TlsError;
use rcgen::{
    BasicConstraints, CertificateParams, CidrSubnet, DnType, ExtendedKeyUsagePurpose,
    GeneralSubtree, IsCa, Issuer, KeyPair, KeyUsagePurpose, NameConstraints,
};
use std::{
    fs,
    io::Write,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
};
use time::{Duration, OffsetDateTime};

pub const CA_CERT_FILE: &str = "ca.pem";
const CA_KEY_FILE: &str = "ca.key";
const SERVER_CERT_FILE: &str = "server.pem";
const SERVER_KEY_FILE: &str = "server.key";
/// What the stored server certificate was issued for.
const SERVER_STATE_FILE: &str = "server.state";

const CA_LIFETIME: Duration = Duration::days(3650);
/// Apple devices reject server certificates that are valid for longer.
const SERVER_LIFETIME: Duration = Duration::days(397);
const RENEW_BEFORE_EXPIRY: Duration = Duration::days(30);
/// Allowance for devices whose clock is behind the server's.
const BACKDATE: Duration = Duration::days(2);

/// The only addresses the authority may certify: loopback and local networks.
/// A stolen authority key thus cannot impersonate sites on the internet.
const PERMITTED_V4: [([u8; 4], u8); 6] = [
    ([10, 0, 0, 0], 8),
    ([100, 64, 0, 0], 10),
    ([127, 0, 0, 0], 8),
    ([169, 254, 0, 0], 16),
    ([172, 16, 0, 0], 12),
    ([192, 168, 0, 0], 16),
];
/// Unique local addresses and the loopback address.
const PERMITTED_V6: [(Ipv6Addr, u8); 2] = [
    (Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7),
    (Ipv6Addr::LOCALHOST, 128),
];
const PERMITTED_DOMAINS: [&str; 2] = ["local", "localhost"];

/// The names and addresses the server certificate is valid for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerNames {
    domains: Vec<String>,
    addresses: Vec<IpAddr>,
}

impl ServerNames {
    /// `<hostname>.local`, `localhost`, the loopback addresses and those of
    /// `addresses` that belong to a local network.
    pub fn new(hostname: &str, addresses: impl IntoIterator<Item = IpAddr>) -> Self {
        let host = hostname.split('.').next().unwrap_or_default();
        let host = host.to_ascii_lowercase();
        let mut domains = vec!["localhost".to_owned()];
        if !host.is_empty() && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            domains.insert(0, format!("{host}.local"));
        }
        let mut addresses: Vec<IpAddr> = addresses
            .into_iter()
            .chain([Ipv4Addr::LOCALHOST.into(), Ipv6Addr::LOCALHOST.into()])
            .filter(|address| is_local(*address))
            .collect();
        addresses.sort_unstable();
        addresses.dedup();
        Self { domains, addresses }
    }

    pub fn all(&self) -> Vec<String> {
        let addresses = self.addresses.iter().map(IpAddr::to_string);
        self.domains.iter().cloned().chain(addresses).collect()
    }
}

fn is_local(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(v4) => PERMITTED_V4.iter().any(|&(network, prefix)| {
            let mask = u32::MAX << (32 - u32::from(prefix));
            u32::from(v4) & mask == u32::from(Ipv4Addr::from(network)) & mask
        }),
        IpAddr::V6(v6) => PERMITTED_V6.iter().any(|&(network, prefix)| {
            let mask = u128::MAX << (128 - u32::from(prefix));
            u128::from(v6) & mask == u128::from(network) & mask
        }),
    }
}

/// A server certificate with the chain up to the authority, as PEM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issued {
    pub chain_pem: String,
    pub key_pem: String,
}

/// The certificate authority stored in one directory.
#[derive(Debug)]
pub struct LocalCa {
    directory: PathBuf,
}

impl LocalCa {
    /// Uses the authority in `directory`, creating it on first use. An
    /// incomplete one is an error: replacing it would break every device that
    /// trusts it.
    pub fn open(directory: &Path, hostname: &str, now: OffsetDateTime) -> Result<Self, TlsError> {
        create_private_directory(directory)?;
        let ca = Self {
            directory: directory.to_owned(),
        };
        let has_cert = ca.path(CA_CERT_FILE).exists();
        let has_key = ca.path(CA_KEY_FILE).exists();
        match (has_cert, has_key) {
            (true, true) => {}
            (false, false) => ca.create(hostname, now)?,
            _ => return Err(TlsError::IncompleteAuthority(directory.to_owned())),
        }
        Ok(ca)
    }

    pub fn certificate_pem(&self) -> Result<String, TlsError> {
        self.read(CA_CERT_FILE)
    }

    /// The stored server certificate if it still covers `names` and is not
    /// about to expire; otherwise a newly issued one.
    pub fn server_certificate(
        &self,
        names: &ServerNames,
        now: OffsetDateTime,
    ) -> Result<Issued, TlsError> {
        if self.stored_is_current(names, now) {
            return Ok(Issued {
                chain_pem: self.read(SERVER_CERT_FILE)?,
                key_pem: self.read(SERVER_KEY_FILE)?,
            });
        }
        self.issue(names, now)
    }

    fn create(&self, hostname: &str, now: OffsetDateTime) -> Result<(), TlsError> {
        let key = KeyPair::generate()?;
        let certificate = authority_params(hostname, now)?.self_signed(&key)?;
        self.write(CA_KEY_FILE, &key.serialize_pem())?;
        self.write(CA_CERT_FILE, &certificate.pem())
    }

    fn issue(&self, names: &ServerNames, now: OffsetDateTime) -> Result<Issued, TlsError> {
        let ca_pem = self.certificate_pem()?;
        let ca_key = KeyPair::from_pem(&self.read(CA_KEY_FILE)?)?;
        let issuer = Issuer::from_ca_cert_pem(&ca_pem, ca_key)?;
        let key = KeyPair::generate()?;
        let certificate = server_params(names, now)?.signed_by(&key, &issuer)?;
        let issued = Issued {
            chain_pem: format!("{}{ca_pem}", certificate.pem()),
            key_pem: key.serialize_pem(),
        };
        // The state goes last: after an interrupted write the certificate is
        // issued again.
        let _ = fs::remove_file(self.path(SERVER_STATE_FILE));
        self.write(SERVER_KEY_FILE, &issued.key_pem)?;
        self.write(SERVER_CERT_FILE, &issued.chain_pem)?;
        self.write(SERVER_STATE_FILE, &state_text(names, now))?;
        Ok(issued)
    }

    fn stored_is_current(&self, names: &ServerNames, now: OffsetDateTime) -> bool {
        let Ok(state) = self.read(SERVER_STATE_FILE) else {
            return false;
        };
        let mut lines = state.lines();
        let issued = lines
            .next()
            .and_then(|line| line.parse().ok())
            .and_then(|seconds| OffsetDateTime::from_unix_timestamp(seconds).ok());
        let Some(issued) = issued else {
            return false;
        };
        lines.eq(names.all().iter().map(String::as_str))
            && now >= issued - BACKDATE
            && now < issued + SERVER_LIFETIME - BACKDATE - RENEW_BEFORE_EXPIRY
    }

    fn path(&self, file: &str) -> PathBuf {
        self.directory.join(file)
    }

    fn read(&self, file: &str) -> Result<String, TlsError> {
        let path = self.path(file);
        fs::read_to_string(&path).map_err(|error| TlsError::File(path, error))
    }

    /// Writes a file only the service user can read, replacing it as a whole.
    fn write(&self, file: &str, content: &str) -> Result<(), TlsError> {
        let path = self.path(file);
        let partial = self.path(&format!("{file}.partial"));
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        options
            .open(&partial)
            .and_then(|mut file| file.write_all(content.as_bytes()))
            .and_then(|()| fs::rename(&partial, &path))
            .map_err(|error| TlsError::File(path, error))
    }
}

fn create_private_directory(directory: &Path) -> Result<(), TlsError> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder
        .create(directory)
        .map_err(|error| TlsError::File(directory.to_owned(), error))
}

fn state_text(names: &ServerNames, issued: OffsetDateTime) -> String {
    let mut text = format!("{}\n", issued.unix_timestamp());
    for name in names.all() {
        text.push_str(&name);
        text.push('\n');
    }
    text
}

fn authority_params(hostname: &str, now: OffsetDateTime) -> Result<CertificateParams, TlsError> {
    let mut params = CertificateParams::new(Vec::new())?;
    params.distinguished_name.push(
        DnType::CommonName,
        format!("Veltrunox local CA ({hostname})"),
    );
    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    params.name_constraints = Some(NameConstraints {
        permitted_subtrees: permitted_subtrees(),
        excluded_subtrees: Vec::new(),
    });
    params.not_before = now - BACKDATE;
    params.not_after = now + CA_LIFETIME;
    Ok(params)
}

fn permitted_subtrees() -> Vec<GeneralSubtree> {
    let domains = PERMITTED_DOMAINS.map(|domain| GeneralSubtree::DnsName(domain.to_owned()));
    let v4 = PERMITTED_V4.map(|(network, prefix)| {
        GeneralSubtree::IpAddress(CidrSubnet::from_v4_prefix(network, prefix))
    });
    let v6 = PERMITTED_V6.map(|(network, prefix)| {
        GeneralSubtree::IpAddress(CidrSubnet::from_v6_prefix(network.octets(), prefix))
    });
    domains.into_iter().chain(v4).chain(v6).collect()
}

fn server_params(names: &ServerNames, now: OffsetDateTime) -> Result<CertificateParams, TlsError> {
    let mut params = CertificateParams::new(names.all())?;
    if let Some(name) = names.domains.first() {
        params
            .distinguished_name
            .push(DnType::CommonName, name.clone());
    }
    params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    params.use_authority_key_identifier_extension = true;
    params.not_before = now - BACKDATE;
    params.not_after = now - BACKDATE + SERVER_LIFETIME;
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    fn names() -> ServerNames {
        ServerNames::new("PinsPlayer", [address("192.168.1.20")])
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap()
    }

    #[test]
    fn names_hold_the_local_name_and_only_local_addresses() {
        let names = ServerNames::new(
            "PinsPlayer.lan",
            [
                address("192.168.1.20"),
                address("8.8.8.8"),
                address("2a02:1::5"),
                address("fe80::1"),
                address("fd00::5"),
            ],
        );
        assert_eq!(
            names.all(),
            [
                "pinsplayer.local",
                "localhost",
                "127.0.0.1",
                "192.168.1.20",
                "::1",
                "fd00::5"
            ]
        );
    }

    #[test]
    fn the_authority_stays_and_the_certificate_is_kept_while_current() {
        let directory = tempfile::tempdir().unwrap();
        let ca = LocalCa::open(directory.path(), "pi", now()).unwrap();
        let authority = ca.certificate_pem().unwrap();
        let first = ca.server_certificate(&names(), now()).unwrap();
        let again = LocalCa::open(directory.path(), "pi", now()).unwrap();
        assert_eq!(again.certificate_pem().unwrap(), authority);
        let later = now() + Duration::days(100);
        assert_eq!(again.server_certificate(&names(), later).unwrap(), first);
    }

    #[test]
    fn new_addresses_nearing_expiry_or_a_clock_set_back_issue_anew() {
        let directory = tempfile::tempdir().unwrap();
        let ca = LocalCa::open(directory.path(), "pi", now()).unwrap();
        let first = ca.server_certificate(&names(), now()).unwrap();
        let moved = ServerNames::new("PinsPlayer", [address("10.0.0.7")]);
        let second = ca.server_certificate(&moved, now()).unwrap();
        assert_ne!(second, first);
        let expiring = now() + Duration::days(370);
        let third = ca.server_certificate(&moved, expiring).unwrap();
        assert_ne!(third, second);
        let set_back = expiring - Duration::days(30);
        assert_ne!(ca.server_certificate(&moved, set_back).unwrap(), third);
    }

    #[test]
    fn half_an_authority_is_not_replaced() {
        let directory = tempfile::tempdir().unwrap();
        LocalCa::open(directory.path(), "pi", now()).unwrap();
        fs::remove_file(directory.path().join(CA_KEY_FILE)).unwrap();
        assert!(matches!(
            LocalCa::open(directory.path(), "pi", now()),
            Err(TlsError::IncompleteAuthority(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn keys_are_readable_by_the_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let ca = LocalCa::open(directory.path(), "pi", now()).unwrap();
        ca.server_certificate(&names(), now()).unwrap();
        for file in [CA_KEY_FILE, SERVER_KEY_FILE] {
            let mode = fs::metadata(directory.path().join(file))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600, "{file}");
        }
    }
}
