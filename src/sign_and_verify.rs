use secp256k1::ecdsa::Signature as SecpSignature;
use secp256k1::hashes::sha256;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Message, Secp256k1};
use secp256k1::{PublicKey as SecpPublicKey, SecretKey as SecpSecretKey};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::str;

#[derive(Clone, Deserialize, Serialize)]
pub struct Signature(SecpSignature);

impl Deref for Signature {
    type Target = SecpSignature;
    fn deref(&self) -> &SecpSignature {
        return &self.0;
    }
}

impl DerefMut for Signature {
    fn deref_mut(&mut self) -> &mut SecpSignature {
        return &mut self.0;
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct PublicKey(pub SecpPublicKey);

impl Deref for PublicKey {
    type Target = SecpPublicKey;
    fn deref(&self) -> &SecpPublicKey {
        return &self.0;
    }
}

impl DerefMut for PublicKey {
    fn deref_mut(&mut self) -> &mut SecpPublicKey {
        return &mut self.0;
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PrivateKey(SecpSecretKey);

impl Deref for PrivateKey {
    type Target = SecpSecretKey;
    fn deref(&self) -> &SecpSecretKey {
        return &self.0;
    }
}

impl DerefMut for PrivateKey {
    fn deref_mut(&mut self) -> &mut SecpSecretKey {
        return &mut self.0;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Verifier {}

impl Verifier {
    //We verify whether the hash of a signed transaction correpsonds to the public key passed as a parameter
    pub fn verify(
        &self,
        message: &str,
        signed_message: &Signature,
        public_key: &PublicKey,
    ) -> bool {
        let secp = Secp256k1::new();
        let message = Message::from_hashed_data::<sha256::Hash>(&message.as_bytes());
        return secp
            .verify_ecdsa(&message, &signed_message, &public_key)
            .is_ok();
    }
}

//We sign a message and return its signed hash + the public key that was generated
pub fn sign(message: &str, private_key: &PrivateKey) -> Signature {
    let secp = Secp256k1::new();
    let message = Message::from_hashed_data::<sha256::Hash>(&message.as_bytes());
    let signed_message = secp.sign_ecdsa(&message, &private_key);
    return Signature(signed_message);
}

pub fn create_keypair() -> (PrivateKey, PublicKey) {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    return (PrivateKey(secret_key), PublicKey(public_key));
}

#[cfg(test)]
mod tests {
    use crate::hash::hash_as_string;

    use super::*;

    #[test]
    fn test_verify_signature() {
        let verifier = Verifier {};
        let transaction_hash: String = hash_as_string([String::from("a")].last().unwrap());
        let (private_key, public_key) = create_keypair();
        let signature_of_sender = sign(&transaction_hash, &private_key);

        assert_eq!(
            true,
            verifier.verify(&transaction_hash, &signature_of_sender, &public_key)
        );
    }
}
