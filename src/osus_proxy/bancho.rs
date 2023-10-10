use std::io::{self, Read};

use bytebuffer::{ByteBuffer, Endian};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use strum::{Display, EnumIter};

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

#[repr(u8)]
#[derive(Debug, PartialEq, FromPrimitive, ToPrimitive)]
pub enum UserAction {
    Idle = 0,
    Afk = 1,
    Playing = 2,
    Editing = 3,
    Modding = 4,
    Multiplayer = 5,
    Watching = 6,
    Unknown = 7,
    Testing = 8,
    Submitting = 9,
    Paused = 10,
    Lobby = 11,
    Multiplaying = 12,
    OsuDirect = 13,
}

impl UserAction {
    pub fn as_u8(&self) -> u8 {
        ToPrimitive::to_u8(self).expect("How do we even have a self of this...")
    }

    pub fn from_u8(repr: u8) -> Self {
        FromPrimitive::from_u8(repr).unwrap_or(Self::Unknown)
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Display, FromPrimitive, ToPrimitive, EnumIter)]
pub enum Country {
    Unknown = 0,
    UnitedArabEmirates = 4,
    Argentina = 13,
    Austria = 15,
    Australia = 16,
    Azerbaijan = 18,
    Barbados = 20,
    Bangladesh = 21,
    Belgium = 22,
    Bulgaria = 24,
    Bahrain = 25,
    Brunei = 29,
    Brazil = 31,
    Bhutan = 33,
    Botswana = 35,
    Belarus = 36,
    Canada = 38,
    Switzerland = 43,
    CoteDIvoire = 44,
    Chile = 46,
    China = 48,
    Colombia = 49,
    CostaRica = 50,
    Cuba = 51,
    Cyprus = 54,
    Czechia = 55,
    Germany = 56,
    Djibouti = 57,
    Denmark = 58,
    Algeria = 61,
    Ecuador = 62,
    Estonia = 63,
    Egypt = 64,
    Spain = 67,
    Ethiopia = 68,
    Finland = 69,
    Fiji = 70,
    France = 74,
    Gabon = 76,
    UnitedKingdom = 77,
    Ghana = 81,
    Greece = 88,
    Guam = 91,
    HongKong = 94,
    Honduras = 96,
    Croatia = 97,
    Hungary = 99,
    Indonesia = 100,
    Ireland = 101,
    Israel = 102,
    India = 103,
    Iraq = 105,
    Iran = 106,
    Iceland = 107,
    Italy = 108,
    Jamaica = 109,
    Jordan = 110,
    Japan = 111,
    Kenya = 112,
    Cambodia = 114,
    SouthKorea = 119,
    Kuwait = 120,
    Liechtenstein = 126,
    SriLanka = 127,
    Lithuania = 130,
    Luxembourg = 131,
    Latvia = 132,
    Morocco = 134,
    Monaco = 135,
    Madagascar = 137,
    NorthMacedonia = 139,
    Myanmar = 141,
    Mongolia = 142,
    Malta = 148,
    Mauritius = 149,
    Maldives = 150,
    Mexico = 152,
    Malaysia = 153,
    NewCaledonia = 156,
    Nigeria = 159,
    Netherlands = 161,
    Norway = 162,
    Nepal = 163,
    NewZealand = 166,
    Oman = 167,
    Panama = 168,
    Peru = 169,
    PapuaNewGuinea = 171,
    Philippines = 172,
    Pakistan = 173,
    Poland = 174,
    Portugal = 179,
    Paraguay = 181,
    Qatar = 182,
    Romania = 184,
    RussianFederation = 185,
    SaudiArabia = 187,
    Sudan = 190,
    Sweden = 191,
    Singapore = 192,
    Slovenia = 194,
    Slovakia = 196,
    SierraLeone = 197,
    Senegal = 199,
    ElSalvador = 203,
    SyrianArabRepublic = 204,
    Togo = 209,
    Thailand = 210,
    Tunisia = 214,
    Turkey = 217,
    TrinidadAndTobago = 218,
    Taiwan = 220,
    Tanzania = 221,
    Ukraine = 222,
    UnitedStates = 225,
    Uruguay = 226,
    Venezuela = 230,
    Vietnam = 233,
    SouthAfrica = 240,
    Zimbabwe = 243,
}

impl Country {
    pub fn as_u8(&self) -> u8 {
        ToPrimitive::to_u8(self).expect("How do we even have a self of this...")
    }
}

#[repr(u16)]
#[derive(Debug)]
pub enum BanchoPacket {
    ChangeAction {
        action: UserAction,
        info_text: String,
        map_md5: String,
        // TODO: bitfield
        mods: u32,
        mode: u8,
        map_id: i32,
    } = 0,
    SendPublicMessage(OsuMessage) = 1,
    UserId(i32) = 5,
    SendMessage(OsuMessage) = 7,
    SendPrivateMessage(OsuMessage) = 25,
    Privilege {
        // TODO: bitfield
        privileges_bitfield: u32
    } = 71,
    UserPresence {
        user_id: i32,
        name: String,
        utc_offset: u8,
        country_code: u8,
        bancho_privileges: u8,
        longitude: f32,
        latitude: f32,
        global_rank: i32,
    } = 83,
    Other { id: u16, data: Vec<u8> } = u16::MAX,
}

impl BanchoPacket {
    pub fn from_header_and_bytebuf(
        header: &BanchoPacketHeader,
        bytebuf: &mut ByteBuffer,
    ) -> io::Result<Self> {
        match header.id {
            0 => {
                let action = bytebuf.read_u8()?;
                let action = UserAction::from_u8(action);
                let info_text = bytebuf.read_osu_string()?;
                let map_md5 = bytebuf.read_osu_string()?;
                let mods = bytebuf.read_u32()?;
                let mode = bytebuf.read_u8()?;
                let map_id = bytebuf.read_i32()?;
                Ok(Self::ChangeAction {
                    action,
                    info_text,
                    map_md5,
                    mods,
                    mode,
                    map_id,
                })
            }
            1 => {
                let message = bytebuf.read_osu_message()?;
                Ok(Self::SendPublicMessage(message))
            }
            5 => {
                let user_id = bytebuf.read_i32()?;
                Ok(Self::UserId(user_id))
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
            83 => {
                let user_id = bytebuf.read_i32()?;
                let name = bytebuf.read_osu_string()?;
                let utc_offset = bytebuf.read_u8()?;
                let country_code = bytebuf.read_u8()?;
                let bancho_privileges = bytebuf.read_u8()?;
                let longitude = bytebuf.read_f32()?;
                let latitude = bytebuf.read_f32()?;
                let global_rank = bytebuf.read_i32()?;
                Ok(Self::UserPresence {
                    user_id,
                    name,
                    utc_offset,
                    country_code,
                    bancho_privileges,
                    longitude,
                    latitude,
                    global_rank,
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
            BP::ChangeAction { .. } => 0,
            BP::SendPublicMessage(_) => 1,
            BP::UserId(_) => 5,
            BP::SendMessage(_) => 7,
            BP::SendPrivateMessage(_) => 25,
            BP::Privilege { .. } => 71,
            BP::UserPresence { .. } => 83,
            BP::Other { id, .. } => *id,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        use BanchoPacket as BP;

        let mut bytebuf = ByteBuffer::new();
        bytebuf.set_endian(Endian::LittleEndian);

        match self {
            BP::ChangeAction {
                action,
                info_text,
                map_md5,
                mods,
                mode,
                map_id
            } => {
                bytebuf.write_u8(action.as_u8());
                bytebuf.write_osu_string(&info_text);
                bytebuf.write_osu_string(&map_md5);
                bytebuf.write_u32(*mods);
                bytebuf.write_u8(*mode);
                bytebuf.write_i32(*map_id);
            }
            BP::SendPublicMessage(message) => {
                bytebuf.write_osu_message(message);
            }
            BP::UserId(user_id) => {
                bytebuf.write_i32(*user_id);
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
            BP::UserPresence {
                user_id,
                name,
                utc_offset,
                country_code,
                bancho_privileges,
                longitude,
                latitude,
                global_rank
            } => {
                bytebuf.write_i32(*user_id);
                bytebuf.write_osu_string(name);
                bytebuf.write_u8(*utc_offset);
                bytebuf.write_u8(*country_code);
                bytebuf.write_u8(*bancho_privileges);
                bytebuf.write_f32(*longitude);
                bytebuf.write_f32(*latitude);
                bytebuf.write_i32(*global_rank);
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
