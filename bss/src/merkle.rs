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
