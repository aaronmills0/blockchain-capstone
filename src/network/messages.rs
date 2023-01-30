use std::collections::HashMap;

use bytes::Bytes;
use mini_redis::Frame;
use serde_json;

pub fn get_peerid_query() -> Frame {
    let command = String::from("00000000");
    let peerid_sender = String::from("0".repeat(32));
    let peerid_receiver = String::from("0".repeat(31) + "1");

    let mut header = command.clone();
    header.push_str(&peerid_sender);
    header.push_str(&peerid_receiver);

    let header_bytes = Bytes::from(header);

    let wrapper_header = Frame::Bulk(header_bytes);
    return Frame::Array(Vec::from([wrapper_header]));
}

// notation for functions that return message type is get_name_response()
pub fn get_peerid_response(next_peerid: u32) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let command = String::from("00000001");
    let peerid_sender = String::from("0".repeat(31) + "1");
    let peerid_receiver_unprocessed = format!("{next_peerid:#034b}");
    let mut peerid_receiver = String::new();
    if let Some(part) = peerid_receiver_unprocessed.get(2..) {
        peerid_receiver = part.to_string();
    }

    let mut header = command.clone();
    header.push_str(&peerid_sender);
    header.push_str(&peerid_receiver);

    let header_bytes = Bytes::from(header);

    let wrapper_header = Frame::Bulk(header_bytes);
    response_vec.push(wrapper_header);

    let peerid = Frame::Bulk(Bytes::from(peerid_receiver));
    response_vec.push(peerid);

    return Frame::Array(response_vec);
}

pub fn get_sockets_query() -> Frame {
    let command = String::from("00000010");
    let peerid_sender = String::from("0".repeat(32));
    let peerid_receiver = String::from("0".repeat(31) + "1");

    let mut header = command.clone();
    header.push_str(&peerid_sender);
    header.push_str(&peerid_receiver);

    let header_bytes = Bytes::from(header);

    let wrapper_header = Frame::Bulk(header_bytes);
    return Frame::Array(Vec::from([wrapper_header]));
}

pub fn get_sockets_response(socketmap: HashMap<u32, String>, receiver_id: u32) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();

    let command = String::from("00000011");
    let peerid_sender = String::from("0".repeat(31) + "1");
    let peerid_receiver_unprocessed = format!("{receiver_id:#034b}");
    let mut peerid_receiver = String::new();
    if let Some(part) = peerid_receiver_unprocessed.get(2..) {
        peerid_receiver = part.to_string();
    }

    let mut header = command.clone();
    header.push_str(&peerid_sender);
    header.push_str(&peerid_receiver);

    let header_bytes = Bytes::from(header);

    let wrapper_header = Frame::Bulk(header_bytes);
    response_vec.push(wrapper_header);

    let payload = Frame::Bulk(Bytes::from(serde_json::to_string(&socketmap).unwrap()));
    response_vec.push(payload);
    return Frame::Array(response_vec);
}
