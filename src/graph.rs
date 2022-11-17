use chrono::Local;
use log::{error, info, warn};
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::{env, fs};

use crate::transaction::TxOut;
use crate::{block::Block, hash};

/**
 * Creates a new dot file in a config folder based on the input configuration file.
 * The dot file includes connected blocks with an autounique index and their header's id.
 * It also includes the transactions within a block. They also have an id and a unique numeric label (per block).
 * The file also contains input / output connections including indices and values.
 */
pub fn create_block_graph(initial_tx_outs: Vec<TxOut>, blockchain: Vec<Block>) {
    let slash = if env::consts::OS == "windows" {
        "\\"
    } else {
        "/"
    };

    if fs::create_dir_all("graphs".to_owned() + slash).is_err() {
        warn!("Failed to create directory! Permissions may be needed.");
    }

    let cwd = std::env::current_dir().unwrap();
    let mut dirpath = cwd.into_os_string().into_string().unwrap();
    dirpath.push_str("/graphs");
    let dir_path = Path::new(&dirpath);

    let date_time = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let filename: &str = &format!("{}.dot", date_time);
    let filepath = dir_path.join(filename);
    let file = File::create(&filepath);
    if file.is_err() {
        error!("Failed to created file {}", filename);
        panic!();
    }

    write_line(&file, "digraph blockchain {");
    write_line(&file, "\trankdir=\"RL\"");
    write_line(&file, "\tcompound=true");
    write_blocks(&file, &blockchain);
    write_edges(&file, initial_tx_outs, &blockchain);
    write_line(&file, "}");

    info!("Generated graph with filename {}", filepath.display());
}

/**
 * Writes the blocks and the transactions inside them to the specified file.
 * Ensures that it is written according to the specified blockchain.
 */
fn write_blocks(file: &Result<File, Error>, blockchain: &[Block]) {
    for (i, block) in blockchain.iter().enumerate() {
        write_line(file, &format!("\tsubgraph cluster{} {{", i));
        write_line(file, &format!("\t\t\"i{}\"[style=invis shape=point]", i));

        for (j, transaction) in block.transactions.iter().enumerate() {
            let mut transaction_string = format!("\t\t\"{}\"", hash::hash_as_string(&transaction));
            transaction_string += &format!("[label=\"t{}: ", j);
            transaction_string += &hash::hash_as_string(&transaction)[..6];
            transaction_string += "...\"]";
            write_line(file, &transaction_string);
        }

        let mut block_string = format!("\t\tlabel=\"block {}\\n", i);
        block_string += &hash::hash_as_string(&block)[..6];
        block_string += "...\"";
        write_line(file, &block_string);
        write_line(file, "\t}");
    }
}

/**
 * Writes the code to connect all the transactions and blocks to the specified file.
 * Ensures that it is written according to the specified blockchain and initial tx outs.
 */
fn write_edges(file: &Result<File, Error>, initial_tx_outs: Vec<TxOut>, blockchain: &[Block]) {
    for (i, block) in blockchain.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let mut block_edge = format!("\t\"i{}\" -> \"i{}\"", i, i - 1);
        block_edge += &format!("[ltail=cluster{} lhead=cluster{} color=green]", i, i - 1);
        write_line(file, &block_edge);

        for transaction in block.transactions.iter() {
            for (j, input) in transaction.tx_inputs.iter().enumerate() {
                let p_txid = &input.outpoint.txid;
                let p_idx = input.outpoint.index;

                let mut edge_string = format!("\t\"{}\" -> \"", hash::hash_as_string(&transaction));
                if *p_txid == "0".repeat(64) {
                    edge_string += &format!("i0\"[label=\"out: {}, in: {}", p_idx, j);
                    edge_string += &format!(", val: {}\"]", initial_tx_outs[p_idx as usize].value);
                } else {
                    edge_string += p_txid;
                    edge_string += &format!("\"[label=\"out: {}, in: {}", p_idx, j);
                    edge_string += &format!(", val: {}\"]", val(blockchain, i + 1, p_txid, p_idx));
                }
                write_line(file, &edge_string);
            }
        }
    }
}

/**
 * Writes a one line message to the specified file.
 */
fn write_line(file: &Result<File, Error>, msg: &str) {
    if file
        .as_ref()
        .unwrap()
        .write_all(format!("{}{}", msg, "\n").as_bytes())
        .is_err()
    {
        error!("Failed to write {} to file.", msg);
        panic!();
    }
}

/**
 * Finds the value of an output input pair based on the transaction id and the tx_out_index;
 */
fn val(blockchain: &[Block], num_blocks: usize, txid: &String, tx_out_idx: u32) -> u32 {
    for block in blockchain.iter().take(num_blocks) {
        let t = block
            .transactions
            .iter()
            .find(|&x| hash::hash_as_string(&x) == *txid);
        if let Some(t_out) = t {
            return t_out.tx_outputs[tx_out_idx as usize].value;
        }
    }
    error!("Could not find transaction with id {}", txid);
    panic!();
}
