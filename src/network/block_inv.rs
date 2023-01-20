use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockInv {
    pub count: u32,
    //The key of the inventory is the code number of the type of object we are storing (transaction or block)
    //The value is the hash of that object
    pub inventory: HashMap<u32, String>,
}
