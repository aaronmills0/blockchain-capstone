use crate::components::transaction::Outpoint;
use crate::utils::sign_and_verify::{PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet(pub Vec<(PrivateKey, PublicKey, Outpoint, u32)>);
