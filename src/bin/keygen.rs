use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

fn main() {
    let signing_key = SigningKey::generate(&mut OsRng);
    let public_key = hex::encode(VerifyingKey::from(&signing_key).to_bytes());
    let private_key = hex::encode(signing_key.to_bytes());
    println!("PUBLIC_KEY=ed25519:{public_key}");
    println!("VALIDATOR_PRIVATE_KEY={private_key}");
    println!("Keep VALIDATOR_PRIVATE_KEY secret and never commit it.");
}
