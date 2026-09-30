use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

pub struct Wallet {
    pub address: String,
    pub public_key: String,
    signing_key: SigningKey,
}

impl Wallet {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let public_key = hex::encode(VerifyingKey::from(&signing_key).to_bytes());
        let address = format!("0x{}", &public_key[..40]);
        Self { address, public_key, signing_key }
    }

    pub fn public_key(&self) -> &str { &self.public_key }

    pub fn sign(&self, message: &[u8]) -> String {
        use ed25519_dalek::Signer;
        hex::encode(self.signing_key.sign(message).to_bytes())
    }
}
