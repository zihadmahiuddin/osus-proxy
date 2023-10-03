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

#[repr(u16)]
#[derive(Debug)]
pub enum BanchoPacket {
    // TODO: bitfield
    Privilege { privileges_bitfield: u32 } = 71,
    Other { id: u16, data: Vec<u8> } = u16::MAX,
}

impl BanchoPacket {
    pub fn from_header_and_bytebuf(
        header: &BanchoPacketHeader,
        bytebuf: &mut ByteBuffer,
    ) -> io::Result<Self> {
        match header.id {
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
            BP::Privilege { .. } => 71,
            BP::Other { id, .. } => *id,
        }
    }

    pub fn length(&self) -> u32 {
        use BanchoPacket as BP;
        match self {
            BP::Privilege { .. } => 4,
            BP::Other { data, .. } => data.len().try_into().unwrap(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        use BanchoPacket as BP;

        let mut bytebuf = ByteBuffer::new();
        bytebuf.set_endian(Endian::LittleEndian);

        match self {
            BP::Privilege {
                privileges_bitfield,
            } => {
                dbg!(privileges_bitfield);
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

        // Header
        bytebuf.write_u16(self.id());
        bytebuf.write_u8(0);
        bytebuf.write_u32(self.length());

        bytebuf.write_bytes(&self.encode());

        bytebuf.into_vec()
    }
}
