use std::vec::Vec;

pub struct Transaction {
    pub senders: Vec<String>,
    //pub sender: String,
    pub receivers: Vec<String>,
    //pub receiver: String,
    pub units: Vec<u128>,
}