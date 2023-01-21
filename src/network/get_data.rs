use crate::network::message_header::MessageHeader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GetData {
    pub message_header: MessageHeader,
    pub count: u32,
    pub data_type: u8,
    // send last has of header of last block/transaction in sender blockchain
    pub starting_header_hash: String,
}
