use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub command_name: String,
    pub payload_size: u32,
    pub checksum: u32,
}
