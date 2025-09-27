use anyhow::Context;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ring::digest::{SHA256, digest};
use rustls::client::WebPkiServerVerifier;
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, RootCertStore};
use std::{net::IpAddr, sync::Arc};
use x509_parser::prelude::*;

#[derive(Clone)]
pub struct TlsSettings {
    pub domain: String,
    pub client_cert_pem: String,
    pub client_key_pem: String,
    pub spki_pins_sha256_b64: Vec<String>,
    pub use_native_roots: bool,
}

pub fn build_rustls_client_config(cfg: &TlsSettings) -> anyhow::Result<Arc<ClientConfig>> {
    if cfg.domain.parse::<IpAddr>().is_ok() {
        anyhow::bail!("RMM_DOMAIN must be a DNS name (not IP)");
    }

    let mut roots = RootCertStore::empty();
    if cfg.use_native_roots {
        let native = rustls_native_certs::load_native_certs().context("load native roots")?;
        for cert in native {
            roots.add(cert).ok();
        }
    } else {
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let cert_chain = load_cert_chain_pem(&cfg.client_cert_pem)?;
    let private_key = load_private_key_pem(&cfg.client_key_pem)?;

    let inner = WebPkiServerVerifier::builder(roots.into()).build()?;
    let pins = decode_b64_pins(&cfg.spki_pins_sha256_b64)?;
    let verifier = Arc::new(PinnedVerifier { inner, pins });
    let ccfg = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_client_auth_cert(cert_chain, private_key)
        .context("attach client cert")?;

    Ok(Arc::new(ccfg))
}

fn load_cert_chain_pem(path: &str) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    use std::fs::File;
    use std::io::BufReader;
    let mut rd = BufReader::new(File::open(path).with_context(|| format!("open {}", path))?);
    let mut out = Vec::new();
    for item in rustls_pemfile::certs(&mut rd) {
        let der = item.with_context(|| format!("read cert from {}", path))?;
        out.push(der);
    }
    if out.is_empty() {
        anyhow::bail!("no certs in {}", path);
    }
    Ok(out)
}

fn load_private_key_pem(path: &str) -> anyhow::Result<PrivateKeyDer<'static>> {
    use std::fs::File;
    use std::io::BufReader;
    let mut rd = BufReader::new(File::open(path).with_context(|| format!("open {}", path))?);

    let key = rustls_pemfile::pkcs8_private_keys(&mut rd)
        .next()
        .ok_or_else(|| anyhow::anyhow!("no PKCS#8 key in {}", path))?
        .context("read pkcs8")?;
    Ok(PrivateKeyDer::Pkcs8(key))
}

fn decode_b64_pins(pins_b64: &[String]) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut pins = Vec::new();
    for p in pins_b64 {
        let bytes = BASE64_STANDARD
            .decode(p)
            .with_context(|| format!("bad base64 pin: {}", p))?;
        if bytes.len() != 32 {
            anyhow::bail!("pin must be 32 bytes (sha256)");
        }
        pins.push(bytes);
    }
    Ok(pins)
}

#[derive(Debug)]
struct PinnedVerifier {
    inner: Arc<WebPkiServerVerifier>,
    pins: Vec<Vec<u8>>,
}

impl ServerCertVerifier for PinnedVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        self.inner
            .verify_server_cert(end_entity, intermediates, server_name, ocsp, now)?;

        let der = end_entity.as_ref();
        let (_rem, cert) = X509Certificate::from_der(der)
            .map_err(|_| rustls::Error::General("x509 parse failed".into()))?;
        let spki_der = cert.tbs_certificate.subject_pki.raw;
        let sum = digest(&SHA256, spki_der);

        let hit = self.pins.iter().any(|p| p.as_slice() == sum.as_ref());
        if !hit {
            return Err(rustls::Error::General("SPKI pin mismatch".into()));
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}
