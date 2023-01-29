use bytes::{Buf, Bytes};

use log::{error, info, warn};
use mini_redis::Frame;
use std::collections::HashMap;

pub struct Decoder;

impl Decoder {
    pub fn unwrap_peerid_response(response: Frame) -> u32 {
        let mut peerid: u32 = u32::MIN;
        let array_maker: Vec<u8>;

        match response {
            Frame::Array(x) => match &x[1] {
                Frame::Bulk(b) => {
                    array_maker = b.to_vec();
                    let s = std::str::from_utf8(&array_maker[..]).expect("invalid utf-8 sequence");
                    peerid = isize::from_str_radix(s, 2).unwrap() as u32;
                }

                _ => warn!("Wrong formatting for response"),
            },

            _ => warn!("Wrong formatting for response"),
        };
        return peerid;
    }

    pub fn unwrap_sockets_response(response: Frame) -> HashMap<u32, String> {
        return HashMap::new();
    }
}
