use serde::Serialize;
use std::vec::Vec;

#[derive(Clone, Debug, Serialize)]
pub struct Transaction {
    pub senders: Vec<String>,
    pub receivers: Vec<String>,
    pub units: Vec<u128>,
}
