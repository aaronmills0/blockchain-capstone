use std::vec::Vec;

pub struct Block{
    pub block_id: u128,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone)]
pub struct Transaction {
    pub senders: Vec<String>,
    //pub sender: String,
    pub receivers: Vec<String>,
    //pub receiver: String,
    pub units: Vec<u128>,
}
