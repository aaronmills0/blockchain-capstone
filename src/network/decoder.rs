use log::warn;
use mini_redis::Frame;
use phf::phf_map;
use serde_json;
use std::collections::HashMap;

use crate::components::transaction::Transaction;

static COMMANDS: phf::Map<&'static str, &'static str> = phf_map! {
    "00000000" => "id_query",
    "00000001" => "id_response",
    "00000010" => "ports_query",
    "00000011" => "ports_response",
    "00000100" => "termination",
    "00000101" => "transaction",
    "00000110" => "ACK",
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

pub fn decode_ports_query(msg: &Frame) -> Vec<String> {
    let mut ports = Vec::new();
    let json: String;
    match msg {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                json = String::from_utf8(b.to_vec()).expect("invalid utf-8 sequence");
                ports = serde_json::from_str(&json).expect("failed to convert from json");
            }

            _ => warn!("Expected byte with ports vector from second element of the frame array"),
        },

        _ => warn!("Expected the frame to be an array"),
    };

    return ports;
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

#[allow(clippy::type_complexity)]
pub fn decode_ports_response(
    ports_frame: Frame,
) -> (
    Option<HashMap<u32, String>>,
    Option<HashMap<String, Vec<String>>>,
) {
    let mut ip_map = None;
    let mut port_map = None;
    let ip_json: String;
    let port_json: String;
    match ports_frame {
        Frame::Array(x) => match &x[1..=2] {
            [Frame::Bulk(ip_bytes), Frame::Bulk(ports_bytes)] => {
                ip_json = String::from_utf8(ip_bytes.to_vec()).expect("invalid utf-8 sequence");
                ip_map = Some(serde_json::from_str(&ip_json).expect("failed to convert from json"));
                port_json =
                    String::from_utf8(ports_bytes.to_vec()).expect("invalid utf-8 sequence");
                port_map =
                    Some(serde_json::from_str(&port_json).expect("failed to convert from json"));
            }

            _ => warn!("Expected second and third elements of the frame array to be bytes of ip and ports respectively"),
        },

        _ => warn!("Expected the frame to be an array"),
    };

    return (ip_map, port_map);
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
