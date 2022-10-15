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
            transaction_signatures: String::from("lalala"),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("a")]),
            units: Vec::from([50]),
            transaction_signatures: String::from("lalala"),
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
            transaction_signatures: String::from("lalala"),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("a")]),
            units: Vec::from([50]),
            transaction_signatures: String::from("lalala"),
        };
        let tx2: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("n"), String::from("m"), String::from("l")]),
            units: Vec::from([5, 35, 10]),
            transaction_signatures: String::from("lalala"),
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
