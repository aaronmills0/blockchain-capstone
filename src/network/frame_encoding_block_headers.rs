use bytes::Bytes;

#[derive(Clone)]
pub struct BlockHead;

impl BlockHead {
    pub fn generate_message_header(
        command: String,
        peerid_sender: String,
        peerid_receiver: String,
        payload_size: String,
    ) -> Bytes {
        let mut concatenated_element = command;
        concatenated_element.push_str(&peerid_sender);
        concatenated_element.push_str(&peerid_receiver);
        concatenated_element.push_str(&payload_size);

        let header = Bytes::from(concatenated_element);
        return header;
    }
}
