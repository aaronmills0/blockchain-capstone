use secp256k1::ecdsa::Signature;
use secp256k1::hashes::sha256;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Message, Secp256k1};
use secp256k1::{PublicKey, SecretKey};
use crate::hash::hash_as_string;

//We sign a message and return its signed hash + the public key that was generated
pub fn sign(message: &str, secret_key: &SecretKey) -> Signature {
    let secp = Secp256k1::new();
    let message = Message::from_hashed_data::<sha256::Hash>(&message.as_bytes());
    let signed_message = secp.sign_ecdsa(&message, &secret_key);
    return signed_message;
}

//We verify whether the hash of a signed transaction correpsonds to the public key passed as a parameter
pub fn verify(message: &str, signed_message: &Signature, public_key: &PublicKey) -> bool {
    let secp = Secp256k1::new();
    let message = Message::from_hashed_data::<sha256::Hash>(&message.as_bytes());
    return secp
        .verify_ecdsa(&message, &signed_message, &public_key)
        .is_ok();
}

pub fn create_keypair() -> (SecretKey, secp256k1::PublicKey) {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    return (secret_key, public_key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature() {
        let transaction_hash: String= hash_as_string([String::from("a")].last().unwrap());
        let (secret_key, public_key) = create_keypair();
        let signature_of_sender = sign(&transaction_hash, &secret_key);

        assert_eq!(true, verify(&transaction_hash, &signature_of_sender, &public_key));
    }
}
