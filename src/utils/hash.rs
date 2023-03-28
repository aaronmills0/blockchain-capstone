use std::sync::{mpsc::Sender, Arc};

use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn hash<T: Serialize>(obj: &T) -> [u8; 32] {
    let bytes: Vec<u8> = bincode::serialize(obj).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(bytes);

    let mut byte_slice: [u8; 32] = <[u8; 32]>::default();
    byte_slice.copy_from_slice(&hasher.finalize());
    return byte_slice;
}

pub fn hash_as_string<T: Serialize>(obj: &T) -> String {
    return bytes_to_string(&hash(obj));
}

pub fn hash_parallel_vec<T: Serialize>(sender: Sender<Vec<String>>, vec: &Arc<Vec<T>>) {
    sender.send(vec.iter().map(|x| hash_as_string(&x)).collect());
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
