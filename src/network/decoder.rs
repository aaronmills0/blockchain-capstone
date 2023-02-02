use log::warn;
use mini_redis::Frame;
use phf::phf_map;
use serde_json;
use std::collections::HashMap;

use crate::components::transaction::Transaction;

static COMMANDS: phf::Map<&'static str, &'static str> = phf_map! {
    "00000000" => "id_query",
    "00000001" => "id_response",
    "00000010" => "sockets_query",
    "00000011" => "sockets_response",
    "00000100" => "termination",
    "00000101" => "transaction"
};

pub fn decode_command(msg: &Frame) -> (String, u32, u32) {
    let mut cmd: String = String::new();
    let mut sourceid: u32 = 0;
    let mut destid: u32 = 0;
    let array_maker: Vec<u8>;
    match msg {
        Frame::Array(x) => match &x[0] {
            Frame::Bulk(b) => {
                array_maker = b.to_vec();
                let cmd_bits =
                    std::str::from_utf8(&array_maker[..8]).expect("invalid utf-8 sequence");

                let sourceid_bits =
                    std::str::from_utf8(&array_maker[8..40]).expect("invalid utf-8 sequence");
                sourceid = isize::from_str_radix(sourceid_bits, 2).unwrap() as u32;
                let destid_bits =
                    std::str::from_utf8(&array_maker[40..72]).expect("invalid utf-8 sequence");
                destid = isize::from_str_radix(destid_bits, 2).unwrap() as u32;
                if !COMMANDS.contains_key(cmd_bits) {
                    warn!("command not found");
                } else {
                    cmd = COMMANDS[cmd_bits].to_owned();
                }
            }
            _ => warn!("Wrong formatting for response"),
        },

        _ => warn!("Wrong formatting for response"),
    };
    return (cmd, sourceid, destid);
}

pub fn decode_sockets_query(msg: &Frame) -> Option<String> {
    let mut socket = None;
    let array_maker: Vec<u8>;
    match msg {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                array_maker = b.to_vec();
                let s = std::str::from_utf8(&array_maker[..]).expect("invalid utf-8 sequence");
                socket = Some(String::from(s));
            }

            _ => warn!("Wrong formatting for response"),
        },

        _ => warn!("Wrong formatting for response"),
    };
    return socket;
}

pub fn decode_peerid_response(response: Frame) -> Option<u32> {
    let mut peerid: Option<u32> = None;
    let array_maker: Vec<u8>;

    match response {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                array_maker = b.to_vec();
                let s = std::str::from_utf8(&array_maker[..]).expect("invalid utf-8 sequence");
                peerid = Some(isize::from_str_radix(s, 2).unwrap() as u32);
            }

            _ => warn!("Wrong formatting for response"),
        },

        _ => warn!("Wrong formatting for response"),
    };
    return peerid;
}

pub fn decode_sockets_response(response: Frame) -> Option<HashMap<u32, String>> {
    let mut socketsmap: Option<HashMap<u32, String>> = None;
    let array_maker: Vec<u8>;
    let json: String;
    match response {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                array_maker = b.to_vec();
                json = String::from_utf8(array_maker).expect("invalid utf-8 sequence");
                socketsmap =
                    Some(serde_json::from_str(&json).expect("failed to convert from json"));
            }

            _ => warn!("Wrong formatting for response"),
        },

        _ => warn!("Wrong formatting for response"),
    };
    return socketsmap;
}

pub fn decode_transactions_msg(msg: Frame) -> Option<Transaction> {
    let mut tx = None;
    let array_maker: Vec<u8>;
    let json: String;
    match msg {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                array_maker = b.to_vec();
                json = String::from_utf8(array_maker).expect("invalid utf-8 sequence");
                tx = Some(serde_json::from_str(&json).expect("failed to convert from json"));
            }

            _ => warn!("Wrong formatting for response"),
        },

        _ => warn!("Wrong formatting for response"),
    };
    return tx;
}
