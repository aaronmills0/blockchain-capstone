use log::warn;
use mini_redis::Frame;
use phf::phf_map;
use serde_json;
use std::collections::HashMap;

use crate::components::block::Block;

static COMMANDS: phf::Map<&'static str, &'static str> = phf_map! {
    "00000000" => "id_query",
    "00000001" => "id_response",
    "00000010" => "maps_query",
    "00000011" => "maps_response",
    "00000100" => "termination",
    "00000101" => "transaction",
    "00000110" => "BD_query",
    "00000111" => "BD_response",
    "00001000" => "block"
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

pub fn decode_ports(msg: &Frame) -> Vec<String> {
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
pub fn decode_maps_response(
    maps_frame: Frame,
) -> (
    Option<HashMap<u32, String>>,
    Option<HashMap<String, Vec<String>>>,
) {
    let mut ip_map = None;
    let mut ports_map = None;
    let ip_map_json: String;
    let ports_map_json: String;
    match maps_frame {
        Frame::Array(x) => match &x[1..=2] {
            [Frame::Bulk(ip_map_bytes), Frame::Bulk(ports_map_bytes)] => {
                ip_map_json = String::from_utf8(ip_map_bytes.to_vec()).expect("invalid utf-8 sequence");
                ip_map = Some(serde_json::from_str(&ip_map_json).expect("failed to convert from json"));
                ports_map_json =
                    String::from_utf8(ports_map_bytes.to_vec()).expect("invalid utf-8 sequence");
                ports_map =
                    Some(serde_json::from_str(&ports_map_json).expect("failed to convert from json"));
            }

            _ => warn!("Expected second and third elements of the frame array to be bytes of ip_map and ports_map respectively"),
        },

        _ => warn!("Expected the frame to be an array"),
    };

    return (ip_map, ports_map);
}

pub fn decode_json_msg(msg: Frame) -> Option<String> {
    let array_maker: Vec<u8>;
    let mut json = None;
    match msg {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                array_maker = b.to_vec();
                json = Some(String::from_utf8(array_maker).expect("invalid utf-8 sequence"));
            }

            _ => warn!("Expected second element to be a byte array representing the transaction"),
        },

        _ => warn!("Expected the frame to be an array"),
    };
    return json;
}

pub fn decode_head_hash(msg: Frame) -> Option<String> {
    let mut head_hash: Option<String> = None;
    match msg {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                head_hash = Some(String::from_utf8(b.to_vec()).expect("invalid utf-8 sequence"));
            }

            _ => warn!("Expected bytes with hash of the head of the peer's existing blockchain as the second frame of the frame array"),
        },

        _ => warn!("Expected the frame to be an array"),
    };

    return head_hash;
}

pub fn decode_bd_response(response: Frame) -> Vec<Block> {
    let mut blocks = Vec::new();
    match response {
        Frame::Array(x) => match &x[1] {
            Frame::Bulk(b) => {
                let blocks_json = String::from_utf8(b.to_vec()).expect("invalid utf-8 sequence");
                blocks = serde_json::from_str(&blocks_json).unwrap();
            }

            _ => warn!("Expected bytes with hash of the head of the peer's existing blockchain as the second frame of the frame array"),
        },

        _ => warn!("Expected the frame to be an array"),
    };

    return blocks;
}
