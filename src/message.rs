use std::convert::TryInto;

#[repr(C,packed)]
struct Packet {
    ver: u32,
    // type_: [u8; 4],
    topic: [u8; 40],
    product: [u8; 8],
    ac: [u8; 95],
    position: f64,
}

impl From<&[u8]> for Packet {
    fn from(bytes: &[u8]) -> Self {
        unsafe { std::ptr::read(bytes.as_ptr() as *const Self) }
    }
}

#[derive(Debug)]
pub struct MessagePosition {
    pub ac: String,
    pub ps: f64,
    pub symbol: String,
}

impl Default for MessagePosition {
    fn default() -> Self {
        MessagePosition {
            ac: String::default(),
            ps: f64::default(),
            symbol: String::default(),
        }
    }
}

pub fn decodeMessage(bytes: &[u8]) -> Option<MessagePosition> {
    println!("Packet size: {}",std::mem::size_of::<Packet>());

    if bytes.len() < std::mem::size_of::<Packet>() {
        return None; // 字节数不足，无法解析
    }

    let packet: Packet = match bytes.try_into() {
        Ok(packet) => packet,
        Err(_) => return None, // 字节转换失败，无法解析
    };


    let ac = String::from_utf8_lossy(&packet.ac[..])
        .trim_end_matches('\0')
        .to_string();
    let ps = f64::from_be_bytes(packet.position.to_ne_bytes());
    let symbol = String::from_utf8_lossy(&packet.product[..])
        .trim_end_matches('\0')
        .to_string();

    Some(MessagePosition { ac, ps, symbol })
}

pub fn encodeMessage(message: &MessagePosition) -> [u8; std::mem::size_of::<Packet>()] {
    let mut packet: Packet = Packet {
        ver: 0x15,
        // type_: [0; 4],
        topic: [0; 40],
        product: [0; 8],
        ac: [0; 95],
        position: 0.0,
    };

    let ac_bytes = message.ac.as_bytes();
    let symbol_bytes = message.symbol.as_bytes();

    packet.topic.copy_from_slice(b"0001");
    // packet.topic.copy_from_slice(b"topic");
    packet.product[..symbol_bytes.len()].copy_from_slice(symbol_bytes);
    packet.ac[..ac_bytes.len()].copy_from_slice(ac_bytes);
    packet.position = f64::from_be_bytes(message.ps.to_be_bytes());

    unsafe { std::mem::transmute(packet) }
}

pub fn message_main() {
    let message = MessagePosition {
        ac: String::from("AC123"),
        ps: 123.45,
        symbol: String::from("SYM"),
    };

    let encoded_bytes = encodeMessage(&message);
    let decoded_message = decodeMessage(&encoded_bytes).unwrap();

    println!("Decoded Message:");
    println!("AC: {}", decoded_message.ac);
    println!("PS: {}", decoded_message.ps);
    println!("Symbol: {}", decoded_message.symbol);
}
