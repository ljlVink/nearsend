use rcgen::{CertificateParams, KeyPair};
use sha2::{Digest, Sha256};

/// PEM-encoded certificate and private key pair.
#[derive(Clone, Debug)]
pub struct CertPair {
    pub cert_pem: String,
    pub private_key_pem: String,
    pub certificate_fingerprint: String,
}

/// Generate a self-signed certificate for LocalSend TLS communication.
pub fn generate_self_signed_cert() -> anyhow::Result<CertPair> {
    let key_pair = KeyPair::generate()?;
    let params = CertificateParams::new(vec!["localsend".to_string()])?;
    let cert = params.self_signed(&key_pair)?;
    let fingerprint = hash_certificate_der(cert.der().as_ref());

    Ok(CertPair {
        cert_pem: cert.pem(),
        private_key_pem: key_pair.serialize_pem(),
        certificate_fingerprint: fingerprint,
    })
}

fn hash_certificate_der(der: &[u8]) -> String {
    let digest = Sha256::digest(der);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{:02x}", byte);
    }
    out
}
