#[cfg(test)]
mod tests {
    use crate::hash;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::Transaction;
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};
    use crate::utxo::UTXO;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    fn thoughput() {}
}
