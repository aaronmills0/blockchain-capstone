use std::vec::Vec;
use serde::{Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Transaction {
    pub senders: Vec<String>,
    //pub sender: String,
    pub receivers: Vec<String>,
    //pub receiver: String,
    pub units: Vec<u128>,
    //pub units: u128
    pub transaction_signatures: Vec<String>,
}