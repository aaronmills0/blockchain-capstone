use crate::network::message_header::MessageHeader;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub message_header: MessageHeader,
    pub inventory_type: u8,
    pub count: u32,
    pub inventory: Vec<String>,
}
