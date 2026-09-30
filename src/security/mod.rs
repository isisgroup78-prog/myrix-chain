use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Default)]
pub struct AuditReport { pub passed: bool, pub issues: Vec<String> }

impl AuditReport {
    pub fn new() -> Self { Self { passed: true, issues: Vec::new() } }
    pub fn add_issue(&mut self, issue: String) { self.issues.push(issue); self.passed = false; }
}

#[derive(Clone, Debug, Default)]
pub struct SecurityAudit { pub static_checks: bool, pub fuzzed: bool, pub formal_verified: bool }

impl SecurityAudit {
    pub fn new() -> Self { Self { static_checks: true, fuzzed: false, formal_verified: false } }
}

pub fn verify_ed25519(public_key_hex: &str, message: &[u8], signature_hex: &str) -> Result<(), String> {
    let pk = hex::decode(public_key_hex.strip_prefix("ed25519:").unwrap_or(public_key_hex)).map_err(|_| "invalid public key")?;
    let sig = hex::decode(signature_hex).map_err(|_| "invalid signature")?;
    let pk: [u8;32] = pk.try_into().map_err(|_| "public key must be 32 bytes")?;
    let sig: [u8;64] = sig.try_into().map_err(|_| "signature must be 64 bytes")?;
    let key = VerifyingKey::from_bytes(&pk).map_err(|_| "invalid public key")?;
    let signature = Signature::from_bytes(&sig);
    ed25519_dalek::Verifier::verify(&key, message, &signature).map_err(|_| "signature verification failed".to_string())
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}
