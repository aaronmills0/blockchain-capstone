pub struct Parser;
use log::{info, warn};
use std::collections::HashMap;

impl Parser {
    pub fn extract_command(tx_headers: String) {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(String::from("000"), String::from("tx_headers"));

        let command: String = tx_headers[..3].to_string();

        match map.get(&command) {
            Some(vl_0) => {
                info!("The command is: {}", vl_0);
            }
            None => {
                warn!("Command not found.");
            }
        }
    }

    pub fn extract_header(tx_headers: String) {
        let previous_hash: String = tx_headers[3..67].to_string();
    }

    pub fn extract_merkle_root(tx_headers: String) {
        let merkle_root: String = tx_headers[67..131].to_string();
    }

    pub fn extract_nonce(tx_headers: String) {
        let nonce: String = tx_headers[131..139].to_string();
    }
}
