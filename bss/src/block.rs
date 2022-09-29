use std::vec::Vec;
mod transaction;
use transaction::Transaction;

pub struct block{
    pub block_id: u64,
    pub transactions: Vec<Transaction> 
}