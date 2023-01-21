use crate::network::message_header::MessageHeader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GetHeader {
    pub message_header: MessageHeader,
    pub count: u32,
    // send last has of header of last block in sender blockchain
    pub starting_header_hash: String,
}
