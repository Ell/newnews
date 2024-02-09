use crate::error::Error;
use crate::stream::{MaybeTlsStream, Mode};
use tokio::io::{AsyncRead, AsyncWrite};

#[non_exhaustive]
#[derive(Clone)]
pub enum Connector {
    /// Plain (Non-TLS) connector
    Plain,
    /// `native-tls` connector
    #[cfg(feature = "native-tls")]
    NativeTls(native_tls_crate::TlsConnector),
    /// `rustls` connector
    #[cfg(feature = "__rustls-tls")]
    Rustls(std::sync::Arc<rustls::ClientConfig>),
}

mod encryption {
    #[cfg(feature = "native-tls")]
    pub mod native_tls {
        use native_tls_crate::TlsConnector;
        use tokio_native_tls::TlsConnector as TokioTlsConnector;

        use tokio::io::{AsyncRead, AsyncWrite};

        use crate::error::{Error, TlsError};
        use crate::stream::{MaybeTlsStream, Mode};

        pub async fn wrap_stream<S>(
            socket: S,
            domain: String,
            mode: Mode,
            tls_connector: Option<TlsConnector>,
        ) -> Result<MaybeTlsStream<S>, Error>
        where
            S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
        {
            match mode {
                Mode::Plain => Ok(MaybeTlsStream::Plain(socket)),
                Mode::Tls => {
                    let try_connector = tls_connector.map_or_else(TlsConnector::new, Ok);
                    let connector = try_connector.map_err(TlsError::Native)?;
                    let stream = TokioTlsConnector::from(connector);
                    let connected = stream.connect(&domain, socket).await;

                    match connected {
                        Err(e) => Err(Error::Tls(e.into())),
                        Ok(s) => Ok(MaybeTlsStream::NativeTls(s)),
                    }
                }
            }
        }
    }

    #[cfg(feature = "__rustls-tls")]
    pub mod rustls {
        pub use rustls::ClientConfig;
        use rustls::{RootCertStore, ServerName};
        use tokio_rustls::TlsConnector as TokioTlsConnector;

        use std::{convert::TryFrom, sync::Arc};
        use tokio::io::{AsyncRead, AsyncWrite};

        use crate::{
            error::{Error, TlsError},
            stream::{MaybeTlsStream, Mode},
        };

        pub async fn wrap_stream<S>(
            socket: S,
            domain: String,
            mode: Mode,
            tls_connector: Option<Arc<ClientConfig>>,
        ) -> Result<MaybeTlsStream<S>, Error>
        where
            S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
        {
            match mode {
                Mode::Plain => Ok(MaybeTlsStream::Plain(socket)),
                Mode::Tls => {
                    let config = match tls_connector {
                        Some(config) => config,
                        None => {
                            #[allow(unused_mut)]
                            let mut root_store = RootCertStore::empty();

                            #[cfg(feature = "rustls-tls-native-roots")]
                            {
                                let native_certs = rustls_native_certs::load_native_certs()?;
                                let der_certs: Vec<Vec<u8>> =
                                    native_certs.into_iter().map(|cert| cert.0).collect();
                                let total_number = der_certs.len();
                                let (number_added, number_ignored) =
                                    root_store.add_parsable_certificates(&der_certs);

                                tracing::debug!("Added {number_added}/{total_number} native root certificates (ignored {number_ignored})");
                            }

                            #[cfg(feature = "rustls-tls-webpki-roots")]
                            {
                                root_store.add_trust_anchors(
                                    webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
                                        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                                            ta.subject,
                                            ta.spki,
                                            ta.name_constraints,
                                        )
                                    })
                                );
                            }

                            Arc::new(
                                ClientConfig::builder()
                                    .with_safe_defaults()
                                    .with_root_certificates(root_store)
                                    .with_no_client_auth(),
                            )
                        }
                    };

                    let domain = ServerName::try_from(domain.as_str())
                        .map_err(|_| TlsError::InvalidDnsName)?;
                    let stream = TokioTlsConnector::from(config);
                    let connected = stream.connect(domain, socket).await;

                    match connected {
                        Err(e) => Err(Error::Io(e)),
                        Ok(s) => Ok(MaybeTlsStream::Rustls(s)),
                    }
                }
            }
        }
    }

    pub mod plain {
        use tokio::io::{AsyncRead, AsyncWrite};

        use crate::error::TlsError;
        use crate::{
            error::Error,
            stream::{MaybeTlsStream, Mode},
        };

        pub async fn wrap_stream<S>(socket: S, mode: Mode) -> Result<MaybeTlsStream<S>, Error>
        where
            S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
        {
            match mode {
                Mode::Plain => Ok(MaybeTlsStream::Plain(socket)),
                Mode::Tls => Err(Error::Tls(TlsError::NotEnabled)),
            }
        }
    }
}

pub async fn client_async_tls<S>(
    stream: S,
    domain: String,
    mode: Mode,
    connector: Option<Connector>,
) -> Result<MaybeTlsStream<S>, Error>
where
    S: 'static + AsyncRead + AsyncWrite + Send + Unpin,
    MaybeTlsStream<S>: Unpin,
{
    match connector {
        Some(conn) => match conn {
            #[cfg(feature = "native-tls")]
            Connector::NativeTls(conn) => {
                self::encryption::native_tls::wrap_stream(stream, domain, mode, Some(conn)).await
            }
            #[cfg(feature = "__rustls-tls")]
            Connector::Rustls(conn) => {
                self::encryption::rustls::wrap_stream(stream, domain, mode, Some(conn)).await
            }
            Connector::Plain => self::encryption::plain::wrap_stream(stream, mode).await,
        },
        None => {
            #[cfg(feature = "native-tls")]
            {
                self::encryption::native_tls::wrap_stream(stream, domain, mode, None).await
            }
            #[cfg(all(feature = "__rustls-tls", not(feature = "native-tls")))]
            {
                self::encryption::rustls::wrap_stream(stream, domain, mode, None).await
            }
            #[cfg(not(any(feature = "native-tls", feature = "__rustls-tls")))]
            {
                self::encryption::plain::wrap_stream(stream, mode).await
            }
        }
    }
}
