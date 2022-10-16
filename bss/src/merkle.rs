use serde::Serialize;

use crate::hash::hash_as_string;
use crate::transaction::Transaction;
use std::collections::VecDeque;

#[derive(Serialize)]
pub struct Merkle {
    pub tree: Vec<String>,
}

impl Merkle {
    /**
    * Creates a merkle tree from a list of transactions
    * Uses a queue and a stack to create a merkle tree in array representation
    *
    * Logic:
    *
    * Build one layer of the tree at a time in a bottom-up apprach
    * The queue stores the (pairs of) hashes that have yet to be hashed into their parents
    *
    * The stack is filled after a pair of hashes have been hashed to form their parent
    * We use the stack to reverse the set of hashes for a given and obtain the correct ordering
    * for our array implementation
    *
    * Elements are entered into the merkle tree in reverse order to prevent inefficent insertion
    * into the front of a vector. Instead, the vector is reversed after the construction of the
    * reversed array is complete
    *
    * We start by loading the queue with the hashes of all the transactions.

    * The queue represents a level in the tree bottom-up

    * for each level in the tree (while the queue.size > 1 i.e not at the root yet):

    *  Ensure that we have an even number of hashes

    *  Then for each pair of hashes we:

    *      pop them from the queue

    *      put the hash of their concatenation into the queue

    *      push them onto the stack

    *  Unload the stack into the merkle tree vector
    * Reverse the merkle tree vector because we interted everything in reverse for efficiency reasons
    *
    *  Let h_i be the hash of transaction Txi
    *
    *  Let h_ij be the hash of the concatenation of the hashes of transactions Txi and Txj
    *
    *  Then, for a transaction list: Tx0, Tx1, Tx2, Tx3, Tx4 we expect the following tree
    *
    *                          h_01234444
    *                      /                \
    *                h_0123                  h_4444
    *              /        \              /        \
    *          h_01          h_23      h_44          h_44
    *         /    \        /    \    /    \
    *       h_0     h_1   h_2    h_3 h_4   h_4
    *
    *  h_01234444 is the merkle root of this tree
    *
    */
    pub fn create_merkle_tree(transactions: &Vec<Transaction>) -> Merkle {
        let mut merkle_tree: Vec<String> = Vec::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut stack: VecDeque<String> = VecDeque::new();

        // Load the hashes into queue1
        for tx in transactions {
            queue.push_back(hash_as_string(&tx));
        }

        while queue.len() > 1 {
            // If the queue has an odd number of hashes
            if queue.len() % 2 == 1 {
                // Make sure there are an even number of hashes
                let last_hash: String = queue.back().unwrap().clone();
                queue.push_back(last_hash);
            }

            // Remove two at a time and hash their concatenation to form a new hash
            for _ in 1..=queue.len() / 2 {
                let first_hash: String = queue.pop_front().unwrap();
                let second_hash: String = queue.pop_front().unwrap();

                queue.push_back(hash_as_string(&format!("{}{}", first_hash, second_hash)));

                // Add the hashes to the stack
                stack.push_back(first_hash);
                stack.push_back(second_hash);
            }

            while !stack.is_empty() {
                merkle_tree.push(stack.pop_back().unwrap());
            }
        }
        merkle_tree.push(queue.pop_front().unwrap());
        merkle_tree.reverse();
        let merkle: Merkle = Merkle { tree: merkle_tree };
        return merkle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_merkle_tree_even_number_of_transactions() {
        let tx0: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("x"), String::from("y")]),
            units: Vec::from([20, 30]),
            transaction_signature: String::from("aklsdfjaklladsflajks"),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("a")]),
            units: Vec::from([50]),
            transaction_signature: String::from("aklsdfjaklladsflajks"),
        };

        let h0: String = hash_as_string(&tx0);
        let h1: String = hash_as_string(&tx1);
        let root_hash: String = hash_as_string(&format!("{}{}", h0, h1));

        let transactions: Vec<Transaction> = Vec::from([tx0, tx1]);
        let merkle: Merkle = Merkle::create_merkle_tree(&transactions);

        assert_eq!(3, merkle.tree.len());
        assert_eq!(root_hash, merkle.tree[0]);
        assert_eq!(h0, merkle.tree[1]);
        assert_eq!(h1, merkle.tree[2]);
    }

    #[test]
    fn test_create_merkle_tree_odd_number_of_transactions() {
        let tx0: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("x"), String::from("y")]),
            units: Vec::from([20, 30]),
            transaction_signature: String::from("aklsdfjaklladsflajks"),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("a")]),
            units: Vec::from([50]),
            transaction_signature: String::from("aklsdfjaklladsflajks"),
        };
        let tx2: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("n"), String::from("m"), String::from("l")]),
            units: Vec::from([5, 35, 10]),
            transaction_signature: String::from("aklsdfjaklladsflajks"),
        };

        let h0: String = hash_as_string(&tx0);
        let h1: String = hash_as_string(&tx1);
        let h2: String = hash_as_string(&tx2);
        let h01: String = hash_as_string(&format!("{}{}", h0, h1));
        let h22: String = hash_as_string(&format!("{}{}", h2, h2));
        let root_hash: String = hash_as_string(&format!("{}{}", h01, h22));

        let transactions: Vec<Transaction> = Vec::from([tx0, tx1, tx2]);
        let merkle: Merkle = Merkle::create_merkle_tree(&transactions);

        assert_eq!(7, merkle.tree.len());
        assert_eq!(root_hash, merkle.tree[0]);
        assert_eq!(h01, merkle.tree[1]);
        assert_eq!(h22, merkle.tree[2]);
        assert_eq!(h0, merkle.tree[3]);
        assert_eq!(h1, merkle.tree[4]);
        assert_eq!(h2, merkle.tree[5]);
        assert_eq!(h2, merkle.tree[6]);
    }
}
