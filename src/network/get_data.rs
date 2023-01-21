use crate::network::message_header::MessageHeader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GetData {
    pub message_header: MessageHeader,
    pub count: u32,
    // Valid types: 0. Block Header Hash, 1. Transaction Header Hash, 2. Full Block, 3. Full Transaction
    pub data_type: u8,
    // Send last has of header of last block/transaction in sender blockchain
    pub starting_header_hash: String,
}
