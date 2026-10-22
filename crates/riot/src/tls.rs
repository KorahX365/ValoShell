//! TLS setup. The local Riot Client serves a certificate signed by Riot's own CA, which no
//! trust store knows, so ONLY connections to 127.0.0.1 skip certificate validation. Remote
//! hosts are always verified against the OS trust store.

use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};

/// Make `ring` the process crypto provider. Idempotent; call before building any client.
pub fn install_crypto_provider() {
    // Err only means a provider is already installed, which is the desired state.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Client config for the local Riot Client (HTTPS and websocket).
pub fn local_client_config() -> Result<Arc<ClientConfig>, rustls::Error> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = Arc::new(AcceptLocalCertificate {
        provider: provider.clone(),
    });
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Accepts the Riot Client's self-signed certificate but still checks handshake signatures.
#[derive(Debug)]
struct AcceptLocalCertificate {
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for AcceptLocalCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        let algorithms = &self.provider.signature_verification_algorithms;
        rustls::crypto::verify_tls12_signature(message, cert, signature, algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        let algorithms = &self.provider.signature_verification_algorithms;
        rustls::crypto::verify_tls13_signature(message, cert, signature, algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}
