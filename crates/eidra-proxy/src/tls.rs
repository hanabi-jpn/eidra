use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, SanType,
};
use tokio_rustls::rustls::{self, ServerConfig};

/// Certificate Authority used to sign dynamically generated per-domain certificates.
pub struct CaAuthority {
    /// The signing certificate (rebuilt from key for rcgen's signing API).
    signing_cert: rcgen::Certificate,
    ca_key: KeyPair,
    /// The original CA cert DER bytes — this is what the user trusted in their OS.
    /// Used in the TLS chain so clients validate against the trusted cert.
    original_cert_der: Vec<u8>,
    /// Cache of generated server configs keyed by domain name.
    cache: RwLock<HashMap<String, Arc<ServerConfig>>>,
}

impl CaAuthority {
    /// Load the CA certificate and private key from PEM files on disk.
    pub fn load(
        ca_cert_path: &std::path::Path,
        ca_key_path: &std::path::Path,
    ) -> Result<Self, TlsError> {
        let cert_pem = std::fs::read_to_string(ca_cert_path)
            .map_err(|e| TlsError::Io(format!("reading CA cert: {}", e)))?;
        let key_pem = std::fs::read_to_string(ca_key_path)
            .map_err(|e| TlsError::Io(format!("reading CA key: {}", e)))?;

        let ca_key = KeyPair::from_pem(&key_pem)
            .map_err(|e| TlsError::CertGeneration(format!("parsing CA key: {}", e)))?;

        // Parse the original CA cert PEM to extract DER bytes.
        // This preserves the exact cert the user trusted in their OS.
        let original_cert_der = pem_to_der(&cert_pem)?;

        // Rebuild a CA certificate from the key for rcgen's signing API.
        // rcgen 0.13 doesn't expose from_ca_cert_pem, so we reconstruct.
        // The signing cert is only used internally by rcgen — the TLS chain
        // uses original_cert_der to match what the OS trusts.
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, "Eidra Local CA");
            dn.push(DnType::OrganizationName, "Eidra");
            dn
        };

        let signing_cert = ca_params
            .self_signed(&ca_key)
            .map_err(|e| TlsError::CertGeneration(format!("rebuilding CA cert: {}", e)))?;

        Ok(Self {
            signing_cert,
            ca_key,
            original_cert_der,
            cache: RwLock::new(HashMap::new()),
        })
    }

    /// Get or generate a `rustls::ServerConfig` for the given domain.
    ///
    /// Certificates are cached so that repeated requests for the same domain
    /// do not trigger unnecessary key generation.
    pub fn server_config_for_domain(&self, domain: &str) -> Result<Arc<ServerConfig>, TlsError> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().map_err(|_| TlsError::LockPoisoned)?;
            if let Some(cfg) = cache.get(domain) {
                return Ok(Arc::clone(cfg));
            }
        }

        // Generate a new certificate signed by the CA
        let server_config = self.generate_server_config(domain)?;
        let server_config = Arc::new(server_config);

        // Insert into cache (write lock)
        {
            let mut cache = self.cache.write().map_err(|_| TlsError::LockPoisoned)?;
            cache.insert(domain.to_string(), Arc::clone(&server_config));
        }

        Ok(server_config)
    }

    fn generate_server_config(&self, domain: &str) -> Result<ServerConfig, TlsError> {
        let mut params = CertificateParams::default();
        params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, domain);
            dn.push(DnType::OrganizationName, "Eidra MITM Proxy");
            dn
        };
        params.subject_alt_names =
            vec![SanType::DnsName(domain.try_into().map_err(|e| {
                TlsError::CertGeneration(format!("invalid SAN: {}", e))
            })?)];

        let server_key = KeyPair::generate()
            .map_err(|e| TlsError::CertGeneration(format!("generating server key: {}", e)))?;

        let server_cert = params
            .signed_by(&server_key, &self.signing_cert, &self.ca_key)
            .map_err(|e| TlsError::CertGeneration(format!("signing server cert: {}", e)))?;

        let cert_der = rustls::pki_types::CertificateDer::from(server_cert.der().to_vec());
        // Use the ORIGINAL CA cert DER (what the OS trusts), not the rebuilt one
        let ca_der = rustls::pki_types::CertificateDer::from(self.original_cert_der.clone());
        let key_der = rustls::pki_types::PrivateKeyDer::try_from(server_key.serialize_der())
            .map_err(|e| TlsError::CertGeneration(format!("serializing server key: {}", e)))?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der, ca_der], key_der)
            .map_err(|e| TlsError::Rustls(format!("building server config: {}", e)))?;

        Ok(config)
    }
}

/// Parse PEM-encoded certificate and extract DER bytes.
fn pem_to_der(pem_str: &str) -> Result<Vec<u8>, TlsError> {
    // Simple PEM parser: extract base64 between BEGIN/END markers
    let mut in_cert = false;
    let mut b64 = String::new();
    for line in pem_str.lines() {
        if line.contains("BEGIN CERTIFICATE") {
            in_cert = true;
            continue;
        }
        if line.contains("END CERTIFICATE") {
            break;
        }
        if in_cert {
            b64.push_str(line.trim());
        }
    }

    if b64.is_empty() {
        return Err(TlsError::CertGeneration(
            "no CERTIFICATE block found in PEM".to_string(),
        ));
    }

    // Decode base64
    base64_decode(&b64)
        .map_err(|e| TlsError::CertGeneration(format!("invalid base64 in PEM: {}", e)))
}

/// Simple base64 decoder (no external dependency needed).
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in input.chunks(4) {
        let mut buf = [0u8; 4];
        let mut pad = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            if byte == b'=' {
                pad += 1;
                buf[i] = 0;
            } else if let Some(pos) = TABLE.iter().position(|&b| b == byte) {
                buf[i] = pos as u8;
            } else {
                return Err(format!("invalid base64 char: {}", byte as char));
            }
        }
        if chunk.len() >= 2 {
            output.push((buf[0] << 2) | (buf[1] >> 4));
        }
        if chunk.len() >= 3 && pad < 2 {
            output.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if chunk.len() >= 4 && pad < 1 {
            output.push((buf[2] << 6) | buf[3]);
        }
    }
    Ok(output)
}

/// Create a `rustls::ClientConfig` that trusts platform root certificates.
pub fn make_upstream_client_config() -> Result<rustls::ClientConfig, TlsError> {
    let mut root_store = rustls::RootCertStore::empty();

    let native_certs = rustls_native_certs::load_native_certs();
    if native_certs.certs.is_empty() {
        return Err(TlsError::Rustls(
            "no native root certificates found — cannot verify upstream TLS".to_string(),
        ));
    }
    for cert in native_certs.certs {
        root_store
            .add(cert)
            .map_err(|e| TlsError::Rustls(format!("adding root cert: {}", e)))?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("certificate generation error: {0}")]
    CertGeneration(String),

    #[error("rustls error: {0}")]
    Rustls(String),

    #[error("internal lock poisoned")]
    LockPoisoned,
}
