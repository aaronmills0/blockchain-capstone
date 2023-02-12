use bytes::Bytes;
use mini_redis::Frame;
use serde_json;

use crate::components::transaction::Transaction;

pub fn get_header(sourceid: u32, destid: u32, command: String) -> Frame {
    let peerid_source_unprocessed = format!("{sourceid:#034b}");
    let peerid_dest_unprocessed = format!("{destid:#034b}");

    let mut peerid_source = String::new();
    if let Some(part) = peerid_source_unprocessed.get(2..) {
        peerid_source = part.to_string();
    }

    let mut peerid_dest = String::new();
    if let Some(part) = peerid_dest_unprocessed.get(2..) {
        peerid_dest = part.to_string();
    }

    let mut header = command;
    header.push_str(&peerid_source);
    header.push_str(&peerid_dest);

    let header_bytes = Bytes::from(header);

    let header_frame = Frame::Bulk(header_bytes);

    return header_frame;
}

pub fn get_peerid_query() -> Frame {
    let header_frame = get_header(0, 1, String::from("00000000"));
    return Frame::Array(Vec::from([header_frame]));
}

// notation for functions that return message type is get_name_response()
pub fn get_peerid_response(destid: u32) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let header_frame = get_header(1, destid, String::from("00000001"));
    response_vec.push(header_frame);

    let peerid_dest_unprocessed = format!("{destid:#034b}");
    let mut peerid_dest = String::new();
    if let Some(part) = peerid_dest_unprocessed.get(2..) {
        peerid_dest = part.to_string();
    }

    let peerid = Frame::Bulk(Bytes::from(peerid_dest));
    response_vec.push(peerid);

    return Frame::Array(response_vec);
}

pub fn get_ports_query(sourceid: u32, destid: u32, ports: Vec<String>) -> Frame {
    let header_frame = get_header(sourceid, destid, String::from("00000010"));
    let ports_frame = Frame::Bulk(Bytes::from(serde_json::to_string(&ports).unwrap()));
    return Frame::Array(Vec::from([header_frame, ports_frame]));
}

pub fn get_ports_response(
    sourceid: u32,
    destid: u32,
    ip_map_json: String,
    port_map_json: String,
) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let header_frame = get_header(sourceid, destid, String::from("00000011"));
    response_vec.push(header_frame);

    let ip_frame = Frame::Bulk(Bytes::from(ip_map_json));
    let port_frame = Frame::Bulk(Bytes::from(port_map_json));
    response_vec.push(ip_frame);
    response_vec.push(port_frame);
    return Frame::Array(response_vec);
}

pub fn get_termination_msg(sourceid: u32, destid: u32) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let header_frame = get_header(sourceid, destid, String::from("00000100"));

    response_vec.push(header_frame);

    return Frame::Array(response_vec);
}

pub fn get_transaction_msg(sourceid: u32, destid: u32, tx: Transaction) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let header_frame = get_header(sourceid, destid, String::from("00000101"));
    response_vec.push(header_frame);

    let payload = Frame::Bulk(Bytes::from(serde_json::to_string(&tx).unwrap()));
    response_vec.push(payload);
    return Frame::Array(response_vec);
}

/**
 * Pass the hash of the head of the current chain to receive the remainder of the chain
 * Upon initialization, send the hash of the genesis block
 */
pub fn get_bd_query(sourceid: u32, destid: u32, head_hash: String) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let header_frame = get_header(sourceid, destid, String::from("00000110"));
    response_vec.push(header_frame);

    let payload = Frame::Bulk(Bytes::from(head_hash));
    response_vec.push(payload);
    return Frame::Array(response_vec);
}

pub fn get_bd_response(sourceid: u32, destid: u32, blocks_json: String) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let header_frame = get_header(sourceid, destid, String::from("00000111"));
    response_vec.push(header_frame);

    let blocks_frame = Frame::Bulk(Bytes::from(blocks_json));
    response_vec.push(blocks_frame);
    return Frame::Array(response_vec);
}
