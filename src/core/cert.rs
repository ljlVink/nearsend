use rcgen::{CertificateParams, KeyPair};

/// PEM-encoded certificate and private key pair.
#[derive(Clone, Debug)]
pub struct CertPair {
    pub cert_pem: String,
    pub private_key_pem: String,
}

/// Generate a self-signed certificate for LocalSend TLS communication.
pub fn generate_self_signed_cert() -> anyhow::Result<CertPair> {
    let key_pair = KeyPair::generate()?;
    let params = CertificateParams::new(vec!["localsend".to_string()])?;
    let cert = params.self_signed(&key_pair)?;

    Ok(CertPair {
        cert_pem: cert.pem(),
        private_key_pem: key_pair.serialize_pem(),
    })
}
