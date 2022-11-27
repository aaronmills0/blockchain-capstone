use ed25519_dalek::{
    ExpandedSecretKey, Keypair, PublicKey as DalekPublicKey, SecretKey as DalekSecretKey,
    Signature as DalekSignature, SignatureError, Signer, Verifier as DalekVerifer,
};
use rand_2::distributions::Exp;
use rand_2::rngs::OsRng;
use secp256k1::ecdsa::Signature as SecpSignature;
use secp256k1::hashes::sha256;
use secp256k1::{Message, Secp256k1};
use secp256k1::{PublicKey as SecpPublicKey, SecretKey as SecpSecretKey};
use serde::{Deserialize, Serialize};
use sha2::digest::typenum::private::PrivateIntegerAdd;
use std::ops::{Deref, DerefMut};
use std::str;

#[derive(Clone, Deserialize, Serialize)]
pub struct Signature(DalekSignature);

impl Deref for Signature {
    type Target = DalekSignature;
    fn deref(&self) -> &DalekSignature {
        return &self.0;
    }
}

impl DerefMut for Signature {
    fn deref_mut(&mut self) -> &mut DalekSignature {
        return &mut self.0;
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct PublicKey(pub DalekPublicKey);

impl Deref for PublicKey {
    type Target = DalekPublicKey;
    fn deref(&self) -> &DalekPublicKey {
        return &self.0;
    }
}

impl DerefMut for PublicKey {
    fn deref_mut(&mut self) -> &mut DalekPublicKey {
        return &mut self.0;
    }
}

#[derive(Deserialize, Serialize)]
pub struct PrivateKey(pub DalekSecretKey);

impl Deref for PrivateKey {
    type Target = DalekSecretKey;
    fn deref(&self) -> &DalekSecretKey {
        return &self.0;
    }
}

impl Into<ExpandedSecretKey> for &PrivateKey {
    fn into(self) -> ExpandedSecretKey {
        return ExpandedSecretKey::from(&self.0);
    }
}

impl DerefMut for PrivateKey {
    fn deref_mut(&mut self) -> &mut DalekSecretKey {
        return &mut self.0;
    }
}

impl Clone for PrivateKey {
    fn clone(&self) -> Self {
        return PrivateKey(DalekSecretKey::from_bytes(self.as_bytes()).unwrap());
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
        return public_key
            .verify(message.as_bytes(), signed_message)
            .is_ok();
    }
}

//We sign a message and return its signed hash + the public key that was generated
pub fn sign(message: &str, private_key: &PrivateKey, public_key: &PublicKey) -> Signature {
    let expanded: ExpandedSecretKey = private_key.into();
    return Signature(expanded.sign(message.as_bytes(), &public_key.0));
}

pub fn create_keypair() -> (PrivateKey, PublicKey) {
    let mut csprng = OsRng {};
    let keypair: Keypair = Keypair::generate(&mut csprng);
    return (PrivateKey(keypair.secret), PublicKey(keypair.public));
}

#[cfg(test)]
mod tests {

    use crate::hash::hash_as_string;
    use crate::sign_and_verify::{create_keypair, sign, Verifier};

    #[test]
    fn test_verify_signature() {
        let verifier = Verifier {};
        let transaction_hash: String = hash_as_string([String::from("a")].last().unwrap());
        let (private_key, public_key) = create_keypair();
        let signature_of_sender = sign(&transaction_hash, &private_key, &public_key);
        assert!(verifier.verify(&transaction_hash, &signature_of_sender, &public_key));
    }
}
