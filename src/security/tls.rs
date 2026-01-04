use std::sync::Arc;
use tokio_rustls::rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer}};
use tokio_rustls::TlsAcceptor;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;

#[allow(dead_code)]
pub struct TlsConfig {
    pub acceptor: TlsAcceptor,
}

impl TlsConfig {
    pub fn load(cert_path: &str, key_path: &str) -> Result<Self> {
        let certs = load_certs(cert_path)?;
        let key = load_keys(key_path)?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| anyhow::anyhow!("TLS Config Error: {}", e))?;

        let acceptor = TlsAcceptor::from(Arc::new(config));
        Ok(Self { acceptor })
    }
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

fn load_keys(path: &str) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let key = rustls_pemfile::private_key(&mut reader)?
        .ok_or_else(|| anyhow::anyhow!("No private key found"))?;
    Ok(key)
}
