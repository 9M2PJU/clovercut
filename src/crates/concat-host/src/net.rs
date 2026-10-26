// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors

//! Shared network trust and routed GETs for model/audio downloads, catalogue
//! searches and update checks. Specialized agents retain provider-specific
//! redirect behavior. [`get`] reports actionable failure classes ([`Cause`]).
//!
//! An HTTPS connection is trusted when its certificate chains to a root
//! Clovercut ships (Mozilla's list, as webpki-roots carries it) or to a root
//! the system trusts. ureq trusted the shipped list alone, so a network that
//! re-signs HTTPS with a root of its own - a proxy or a GitHub accelerator
//! in China, an antivirus that scans HTTPS, a company's gateway - failed
//! every download with "invalid peer certificate: UnknownIssuer", while
//! the browser on the same machine, which trusts that root, worked (#517).
//! ureq's own `native-certs` swaps the shipped list for the system's rather
//! than adding to it, and Android gives a process no system store to read,
//! so that would have failed every download on a phone instead.

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::route::Route;

/// An agent with these timeouts that trusts the shipped roots and the
/// system's.
pub fn agent(connect: Duration, read: Duration) -> ureq::Agent {
    agent_builder(connect, read).build()
}

/// Builds a shared-TLS agent while allowing provider-specific redirect policies.
pub fn agent_builder(connect: Duration, read: Duration) -> ureq::AgentBuilder {
    ureq::AgentBuilder::new()
        .timeout_connect(connect)
        .timeout_read(read)
        .tls_config(Arc::clone(tls()))
}

/// Trusts only the shipped roots for the update manifest that declares package hashes.
/// A system-installed inspection root must not authenticate both a substituted
/// manifest and its substituted installer. Downloads and provider clients keep
/// their existing shared TLS configuration and redirect policies.
pub fn strict_agent(connect: Duration, read: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(connect)
        .timeout_read(read)
        .tls_config(Arc::clone(shipped_tls()))
        .build()
}

/// A diagnostic URL without credentials, query values or fragments.
pub fn safe_url(value: &str) -> String {
    let Ok(mut url) = url::Url::parse(value) else {
        return "invalid URL".into();
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

/// Which roots a connection trusts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust {
    /// The shipped roots and the system's: for a file whose digest Concat
    /// already knows - a model, whose sums are compiled in, or an
    /// installer or sound checked against a manifest fetched with
    /// [`Trust::Shipped`].
    System,
    /// The shipped roots alone, for the manifests that say what a download
    /// must hash to: the update manifest and the sound list. A root the
    /// system was given by a proxy or an antivirus is enough to re-sign
    /// github.com, and a manifest fetched through it could name a swapped
    /// installer's digest beside the swapped installer. Behind such a
    /// network the update check and the sound list fail, as every download
    /// did before #517; the files they vouch for still come over
    /// [`Trust::System`].
    Shipped,
}

/// Why a request got no answer, in the terms a person can act on. What
/// the operating system said is kept beside it in [`Failure`]; this is the
/// class it falls in, which is the same on every platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// Something on this computer refused Concat a connection before it
    /// left the machine: a firewall, an antivirus's web shield, parental
    /// controls, a VPN's kill switch. Windows says WSAEACCES (10013), Unix
    /// EACCES or EPERM. The browser beside it is usually allowed.
    Blocked,
    /// No route to anywhere: no network, or the adapter is down.
    Offline,
    /// The name could not be looked up.
    Dns,
    /// The proxy the settings name could not be reached or would not
    /// carry the request: not running, a sign-in Concat cannot give, a
    /// policy that turns the host away.
    Proxy,
    /// The certificate was not one Concat trusts: something re-signs
    /// secure connections on the way.
    Certificate,
    /// The connection was cut, as a network that filters traffic cuts it.
    Reset,
    /// The server, or something in front of it, turned the connection away.
    Refused,
    /// Nothing came back in time.
    Timeout,
    /// The server answered, with an error status.
    Status(u16),
    /// Anything else, which the message says in full.
    Other,
}

impl Cause {
    /// The cause as a sentence of English, for the log and for the
    /// messages the engine hands back. A caller can localize the structured
    /// cause independently of this diagnostic.
    pub fn describe(self) -> String {
        match self {
            Cause::Blocked => "something on this computer blocked Clovercut from the network - \
                               a firewall, an antivirus or a VPN; the browser may be allowed \
                               where Clovercut is not"
                .to_owned(),
            Cause::Offline => "this computer is not connected to the internet".to_owned(),
            Cause::Dns => "the server's name could not be looked up".to_owned(),
            Cause::Proxy => "the proxy could not be used".to_owned(),
            Cause::Certificate => {
                "the connection's certificate is not trusted: something on the network \
                 re-signs secure connections"
                    .to_owned()
            }
            Cause::Reset => "the connection was cut off".to_owned(),
            Cause::Refused => "the connection was refused".to_owned(),
            Cause::Timeout => "the server took too long to answer".to_owned(),
            Cause::Status(code) => format!("the server answered HTTP {code}"),
            Cause::Other => "the request failed".to_owned(),
        }
    }

    /// How much a cause says about why everything failed, when sources
    /// failed in different ways: a block on this machine explains every
    /// other failure, and one server's 404 explains none.
    fn weight(self) -> u8 {
        match self {
            Cause::Blocked => 9,
            Cause::Proxy => 8,
            Cause::Certificate => 7,
            Cause::Offline => 6,
            Cause::Dns => 5,
            Cause::Reset => 4,
            Cause::Refused => 3,
            Cause::Timeout => 2,
            Cause::Status(_) => 1,
            Cause::Other => 0,
        }
    }

    /// Of several causes, the one that best explains them all.
    pub fn telling(causes: impl IntoIterator<Item = Cause>) -> Option<Cause> {
        causes.into_iter().max_by_key(|cause| cause.weight())
    }
}

/// A request that got no answer: ureq's error, the class it falls in, and
/// the proxy it went through, if one.
#[derive(Debug)]
pub struct Failure {
    /// What ureq said; a status is the server's response, whole.
    pub error: ureq::Error,
    /// What it means.
    pub cause: Cause,
    /// The proxy the request went through, as a message names it.
    pub proxy: Option<String>,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{}", self.cause.describe())?;
        if let Some(proxy) = &self.proxy {
            write!(out, " (through the proxy {proxy})")?;
        }
        Ok(())
    }
}

/// Send a GET through the configured system/environment route, preserving
/// headers and the requested trust roots. Loopback always goes directly.
pub fn get(
    url: &str,
    connect: Duration,
    read: Duration,
    trust: Trust,
    prepare: &dyn Fn(ureq::Request) -> ureq::Request,
) -> Result<ureq::Response, Box<Failure>> {
    let send = |route: Option<&Route>| {
        prepare(routed_agent(route, connect, read, trust).get(url))
            .call()
            .map_err(|error| {
                let cause = classify(&error, route.is_some());
                Box::new(Failure {
                    error,
                    cause,
                    proxy: route.map(|route| route.shown.clone()),
                })
            })
    };
    let route = crate::route::for_url(url);
    if let Some(route) = &route {
        log::debug!(
            "network: {} through the proxy {}",
            safe_url(url),
            route.shown
        );
    }
    // Respect an explicit proxy: silently retrying directly can bypass the
    // user's routing policy. A failed route is reported, never bypassed.
    send(route.as_ref())
}

/// An agent with these timeouts, through `route` when there is one.
fn routed_agent(
    route: Option<&Route>,
    connect: Duration,
    read: Duration,
    trust: Trust,
) -> ureq::Agent {
    let tls = match trust {
        Trust::System => tls(),
        Trust::Shipped => shipped_tls(),
    };
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(connect)
        .timeout_read(read)
        .tls_config(Arc::clone(tls));
    if let Some(route) = route {
        builder = builder.proxy(route.proxy.clone());
    }
    builder.build()
}

/// The class `error` falls in. `proxied` because a connection refused or a
/// name not found is the proxy's when there is one: it is the only host
/// the machine connects to.
pub fn classify(error: &ureq::Error, proxied: bool) -> Cause {
    use ureq::ErrorKind;

    let kind = match error {
        ureq::Error::Status(code, _) => return Cause::Status(*code),
        ureq::Error::Transport(transport) => transport.kind(),
    };
    match kind {
        ErrorKind::ProxyConnect | ErrorKind::ProxyUnauthorized | ErrorKind::InvalidProxyUrl => {
            return Cause::Proxy;
        }
        ErrorKind::Dns => return if proxied { Cause::Proxy } else { Cause::Dns },
        _ => {}
    }
    // What the operating system said, wherever ureq wrapped it.
    let mut source = std::error::Error::source(error);
    while let Some(inner) = source {
        if let Some(io) = inner.downcast_ref::<std::io::Error>() {
            return match classify_io(io) {
                Cause::Refused if proxied => Cause::Proxy,
                cause => cause,
            };
        }
        source = inner.source();
    }
    Cause::Other
}

/// The class an I/O error falls in: from a connection, or from a body
/// being read.
pub fn classify_io(error: &std::io::Error) -> Cause {
    use std::io::ErrorKind;

    // TLS speaks through I/O errors with rustls's own inside.
    if let Some(tls) = error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<rustls::Error>())
    {
        return match tls {
            rustls::Error::InvalidCertificate(_) | rustls::Error::NoCertificatesPresented => {
                Cause::Certificate
            }
            _ => Cause::Reset,
        };
    }
    match error.kind() {
        // WSAEACCES (10013) on Windows; EACCES and EPERM elsewhere.
        ErrorKind::PermissionDenied => Cause::Blocked,
        ErrorKind::NetworkUnreachable | ErrorKind::HostUnreachable | ErrorKind::NetworkDown => {
            Cause::Offline
        }
        ErrorKind::ConnectionRefused => Cause::Refused,
        ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted | ErrorKind::UnexpectedEof => {
            Cause::Reset
        }
        ErrorKind::TimedOut | ErrorKind::WouldBlock => Cause::Timeout,
        // A name ureq could not resolve for a SOCKS proxy.
        ErrorKind::NotFound => Cause::Dns,
        _ => Cause::Other,
    }
}

/// The TLS configuration [`Trust::Shipped`] shares: the shipped roots only.
fn shipped_tls() -> &'static Arc<rustls::ClientConfig> {
    static CONFIG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let config = rustls::ClientConfig::builder_with_provider(
            rustls::crypto::ring::default_provider().into(),
        )
        .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
        .expect("ring offers cipher suites for TLS 1.2 and 1.3")
        .with_root_certificates(rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth();
        Arc::new(config)
    })
}

/// The TLS configuration every agent shares. Built once: reading the
/// system's store takes a keychain query on macOS and a registry walk on
/// Windows, which is not worth paying per request.
fn tls() -> &'static Arc<rustls::ClientConfig> {
    static CONFIG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();
    CONFIG.get_or_init(|| {
        // ring, as ureq builds its own default with: ureq turns that
        // provider on, and naming it here does not depend on which
        // provider the process installed as its default, if any.
        let config = rustls::ClientConfig::builder_with_provider(
            rustls::crypto::ring::default_provider().into(),
        )
        .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
        .expect("ring offers cipher suites for TLS 1.2 and 1.3")
        .with_root_certificates(roots())
        .with_no_client_auth();
        Arc::new(config)
    })
}

/// The shipped roots, and the system's beside them.
fn roots() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let system = rustls_native_certs::load_native_certs();
    // A store that cannot be read, in whole or part, leaves the shipped
    // roots, which is where every download stood before.
    if let Some(error) = system.errors.first() {
        log::debug!("network: the system's root certificates were not all read: {error}");
    }
    let (added, _) = roots.add_parsable_certificates(system.certs);
    log::debug!("network: {added} of the system's root certificates trusted beside Clovercut's");
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An I/O error as ureq hands it back from a connection.
    fn transport(error: std::io::Error) -> ureq::Error {
        ureq::Error::from(error)
    }

    #[test]
    fn diagnostics_never_keep_signed_queries_or_proxy_credentials() {
        let shown = safe_url(
            "https://user:password@audio.example/videoplayback?sig=private&token=private#private",
        );
        assert_eq!(shown, "https://audio.example/videoplayback");
        let error = ureq::Error::Status(403, ureq::Response::new(403, "Forbidden", "").unwrap());
        let failure = Failure {
            error,
            cause: Cause::Status(403),
            proxy: None,
        };
        assert!(!failure.to_string().contains("private"));
    }

    #[test]
    fn a_failure_is_classed_by_what_the_system_said() {
        use std::io::{Error, ErrorKind};

        // The report: WSAEACCES (10013) on Windows, which is EACCES (13)
        // or EPERM (1) on a Mac or Linux - a firewall or an antivirus.
        #[cfg(windows)]
        let refused_by_this_machine = Error::from_raw_os_error(10013);
        #[cfg(not(windows))]
        let refused_by_this_machine = Error::from_raw_os_error(13);
        assert_eq!(
            classify(&transport(refused_by_this_machine), false),
            Cause::Blocked
        );
        #[cfg(unix)]
        assert_eq!(
            classify(&transport(Error::from_raw_os_error(1)), false),
            Cause::Blocked
        );

        let class = |kind| classify(&transport(Error::from(kind)), false);
        assert_eq!(class(ErrorKind::NetworkUnreachable), Cause::Offline);
        assert_eq!(class(ErrorKind::TimedOut), Cause::Timeout);
        assert_eq!(class(ErrorKind::ConnectionReset), Cause::Reset);
        assert_eq!(class(ErrorKind::ConnectionRefused), Cause::Refused);
        // Refused through a proxy is the proxy: it is the only host the
        // machine connected to, and one left set after it stopped is the
        // usual reason.
        let refused = transport(Error::from(ErrorKind::ConnectionRefused));
        assert_eq!(classify(&refused, true), Cause::Proxy);
        assert_eq!(class(ErrorKind::Other), Cause::Other);

        // A certificate nobody here trusts, as rustls says it through ureq.
        let untrusted = Error::new(
            ErrorKind::InvalidData,
            rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer),
        );
        assert_eq!(classify(&transport(untrusted), false), Cause::Certificate);

        let missing = ureq::Error::Status(
            404,
            ureq::Response::new(404, "Not Found", "").expect("a response"),
        );
        assert_eq!(classify(&missing, false), Cause::Status(404));
    }

    #[test]
    fn a_real_refused_connection_is_classed_as_refused() {
        // A port that was just free: nothing listens on it.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .and_then(|listener| listener.local_addr())
            .expect("a port")
            .port();
        let failure = get(
            &format!("http://127.0.0.1:{port}/model.bin"),
            Duration::from_secs(2),
            Duration::from_secs(2),
            Trust::System,
            &|request| request,
        )
        .expect_err("nothing answers");
        assert_eq!(failure.cause, Cause::Refused);
        assert_eq!(failure.proxy, None, "loopback never goes through a proxy");
    }

    #[test]
    fn the_cause_that_explains_most_is_the_one_told() {
        let told = Cause::telling([Cause::Status(404), Cause::Blocked, Cause::Timeout]);
        assert_eq!(told, Some(Cause::Blocked));
        assert_eq!(
            Cause::telling([Cause::Status(404), Cause::Timeout]),
            Some(Cause::Timeout)
        );
        assert_eq!(Cause::telling([]), None);
        assert!(Cause::Blocked.describe().contains("firewall"));
    }

    /// The shipped roots are always there, whatever the system's store
    /// gave: a machine whose store is empty or unreadable is no worse off
    /// than before.
    #[test]
    fn the_shipped_roots_are_always_trusted() {
        assert!(roots().len() >= webpki_roots::TLS_SERVER_ROOTS.len());
        // The configuration is whole: a connection can be started on it.
        // The name is never looked up.
        let name = rustls::pki_types::ServerName::try_from("github.com").expect("a valid name");
        assert!(rustls::ClientConnection::new(Arc::clone(tls()), name.clone()).is_ok());
        assert!(rustls::ClientConnection::new(Arc::clone(shipped_tls()), name).is_ok());
    }
}
