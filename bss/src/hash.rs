use sha2::{Sha256, Digest};
use serde::{Serialize};

pub fn hash<T: Serialize>(obj: &T) -> [u8; 32] {
    let bytes:Vec<u8> = bincode::serialize(obj).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let mut byte_slice: [u8; 32] = <[u8; 32]>::default();
    byte_slice.copy_from_slice(&hasher.finalize());
    return byte_slice;
}

pub fn hash_as_string<T: Serialize>(obj: &T) -> String {
    return bytes_to_string(&hash(obj)); 
}

pub fn bytes_to_string(bytes: &[u8]) -> String {
    let mut s: String = String::new();

    for byte in bytes {
        // Specifies that we want the byte represented as a 2 hexadigit character
        let x: String = format!("{:02x}", byte);
        s.push_str(&x);
    }
    return s;
}