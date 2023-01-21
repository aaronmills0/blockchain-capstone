use crate::components::block::BlockHeader;
use crate::network::message_header::MessageHeader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Header {
    pub message_header: MessageHeader,
    pub hash_count: u32,
    pub headers: Vec<BlockHeader>,
}
