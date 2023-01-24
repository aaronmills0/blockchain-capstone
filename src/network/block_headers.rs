use serde::{Deserialize, Serialize};
use std::fmt::Binary;

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockHeaders;

impl BlockHeaders {
    pub fn generate_message_header() -> String {
        let command_name = String::from("000");
        return command_name;
    }

    //The maximum count allowed is 2048
    pub fn generate_count(hash_count: u32) -> String {
        let binary_count: String = format!("{hash_count:12b}");
        return binary_count;
    }

    pub fn general_single_header(previous_hash: String, merkle_root: String, nonce: u32) -> String {
        let mut single_header = Self::convert_to_binary_from_hex(&previous_hash).to_owned();
        let merkle_root = Self::convert_to_binary_from_hex(&merkle_root).to_owned();
        //I assume that the nonce has length of 8
        let binary_nonce: String = format!("{nonce:8b}").to_owned();

        single_header.push_str(&merkle_root);
        single_header.push_str(&binary_nonce);

        return single_header;
    }

    pub fn convert_to_binary_from_hex(hex: &str) -> String {
        hex[2..].chars().map(Self::to_binary).collect()
    }

    pub fn to_binary(c: char) -> &'static str {
        match c {
            '0' => "0000",
            '1' => "0001",
            '2' => "0010",
            '3' => "0011",
            '4' => "0100",
            '5' => "0101",
            '6' => "0110",
            '7' => "0111",
            '8' => "1000",
            '9' => "1001",
            'A' => "1010",
            'B' => "1011",
            'C' => "1100",
            'D' => "1101",
            'E' => "1110",
            'F' => "1111",
            _ => "",
        }
    }
}
