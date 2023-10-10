use std::io::{self, Read};

use bytebuffer::{ByteBuffer, Endian};

pub struct BanchoPacketHeader {
    id: u16,
    #[allow(dead_code)]
    unknown: u8,
    length: u32,
}

impl BanchoPacketHeader {
    pub fn from_bytes(bytes: [u8; 7]) -> io::Result<Self> {
        let mut bytebuf = ByteBuffer::from_bytes(&bytes);
        bytebuf.set_endian(Endian::LittleEndian);
        let id = bytebuf.read_u16()?;
        let unknown = bytebuf.read_u8()?;
        let length = bytebuf.read_u32()?;
        Ok(Self {
            id,
            unknown,
            length,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OsuMessage {
    pub sender: String,
    pub text: String,
    pub recipient: String,
    pub sender_id: i32,
}

pub trait OsuReader {
    fn read_uleb128(&mut self) -> io::Result<u64>;
    fn read_osu_string(&mut self) -> io::Result<String>;
    fn read_osu_message(&mut self) -> io::Result<OsuMessage>;
}

pub trait OsuWriter {
    fn write_uleb128(&mut self, value: u64);
    fn write_osu_string(&mut self, value: &str);
    fn write_osu_message(&mut self, value: &OsuMessage);
}

const LEB128_HIGH_ORDER_BIT: u8 = 1 << 7;
impl OsuReader for ByteBuffer {
    fn read_uleb128(&mut self) -> io::Result<u64> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.read_u8()?;

            if shift == 63 && byte > 1 {
                panic!("Integer overflow when reading ULEB128");
            }

            result |= u64::from(byte & !LEB128_HIGH_ORDER_BIT) << shift;

            if byte & LEB128_HIGH_ORDER_BIT == 0 {
                return Ok(result);
            }

            shift += 7;
        }
    }

    fn read_osu_string(&mut self) -> io::Result<String> {
        let exists = self.read_u8()? == 0x0b;

        if !exists {
            return Ok(String::new());
        }

        let str_length = self.read_uleb128()?;

        match String::from_utf8(self.read_bytes(str_length as usize)?) {
            Ok(string_result) => Ok(string_result),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        }
    }

    fn read_osu_message(&mut self) -> io::Result<OsuMessage> {
        let sender = self.read_osu_string()?;
        let text = self.read_osu_string()?;
        let recipient = self.read_osu_string()?;
        let sender_id = self.read_i32()?;
        Ok(
            OsuMessage {
                sender,
                text,
                recipient,
                sender_id,
            }
        )
    }
}

impl OsuWriter for ByteBuffer {
    fn write_uleb128(&mut self, mut value: u64) {
        loop {
            let mut byte = (value as u8) & !LEB128_HIGH_ORDER_BIT;
            value >>= 7;

            if value != 0 {
                byte |= LEB128_HIGH_ORDER_BIT;
            }

            self.write_u8(byte);

            if value == 0 {
                return;
            }
        }
    }

    fn write_osu_string(&mut self, value: &str) {
        let exists = !value.is_empty();
        if !exists {
            self.write_u8(0x00);
        } else {
            self.write_u8(0x0b);
            let bytes = value.as_bytes();
            self.write_uleb128(bytes.len() as u64);
            self.write_bytes(&bytes);
        }
    }

    fn write_osu_message(&mut self, value: &OsuMessage) {
        self.write_osu_string(&value.sender);
        self.write_osu_string(&value.text);
        self.write_osu_string(&value.recipient);
        self.write_i32(value.sender_id);
    }
}

#[repr(u16)]
#[derive(Debug)]
pub enum BanchoPacket {
    // TODO: bitfield
    SendPublicMessage(OsuMessage) = 1,
    SendMessage(OsuMessage) = 7,
    SendPrivateMessage(OsuMessage) = 25,
    Privilege { privileges_bitfield: u32 } = 71,
    Other { id: u16, data: Vec<u8> } = u16::MAX,
}

impl BanchoPacket {
    pub fn from_header_and_bytebuf(
        header: &BanchoPacketHeader,
        bytebuf: &mut ByteBuffer,
    ) -> io::Result<Self> {
        match header.id {
            1 => {
                let message = bytebuf.read_osu_message()?;
                Ok(Self::SendPublicMessage(message))
            }
            7 => {
                let message = bytebuf.read_osu_message()?;
                Ok(Self::SendMessage(message))
            }
            25 => {
                let message = bytebuf.read_osu_message()?;
                Ok(Self::SendPrivateMessage(message))
            }
            71 => {
                let privileges_bitfield = bytebuf.read_u32()?;
                Ok(Self::Privilege {
                    privileges_bitfield,
                })
            }
            _ => {
                let mut data = vec![0; header.length as usize];
                bytebuf.read_exact(&mut data)?;
                Ok(Self::Other {
                    id: header.id,
                    data,
                })
            }
        }
    }

    pub fn id(&self) -> u16 {
        use BanchoPacket as BP;
        match self {
            BP::SendPublicMessage(_) => 1,
            BP::SendMessage(_) => 7,
            BP::SendPrivateMessage(_) => 25,
            BP::Privilege { .. } => 71,
            BP::Other { id, .. } => *id,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        use BanchoPacket as BP;

        let mut bytebuf = ByteBuffer::new();
        bytebuf.set_endian(Endian::LittleEndian);

        match self {
            BP::SendPublicMessage(message) => {
                bytebuf.write_osu_message(message);
            }
            BP::SendMessage(message) => {
                bytebuf.write_osu_message(message);
            }
            BP::SendPrivateMessage(message) => {
                bytebuf.write_osu_message(message);
            }
            BP::Privilege {
                privileges_bitfield,
            } => {
                bytebuf.write_u32(*privileges_bitfield);
            }
            BP::Other { data, .. } => {
                bytebuf.write_bytes(&data);
            }
        }

        bytebuf.into_vec()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytebuf = ByteBuffer::new();
        bytebuf.set_endian(Endian::LittleEndian);

        let data = self.encode();

        // Header
        bytebuf.write_u16(self.id());
        bytebuf.write_u8(0);
        bytebuf.write_u32(data.len() as u32);

        bytebuf.write_bytes(&data);

        bytebuf.into_vec()
    }
}
