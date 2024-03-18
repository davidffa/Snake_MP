#[derive(PartialEq)]
pub enum PacketType {
    Info,
    FoodUpdate,
    DirectionUpdate,
    HeadUpdate,
    SnakeConnect,
    SnakeDisconnect,
    ConnRejected,
}

pub struct PacketBuilder {
    r#type: PacketType,
    buffer: Vec<u8>,
}

impl PacketBuilder {
    pub fn new(r#type: PacketType) -> Self {
        Self {
            r#type,
            buffer: Vec::new()
        }
    }

    pub fn with_capacity(r#type: PacketType, capacity: usize) -> Self {
        Self {
            r#type,
            buffer: Vec::with_capacity(capacity)
        }
    }

    pub fn write(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn write_u16_le(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn build(&self) -> Vec<u8> {
        let packet_type = match self.r#type {
            PacketType::Info => 0x1,
            PacketType::FoodUpdate => 0x2,
            PacketType::DirectionUpdate => 0x3,
            PacketType::HeadUpdate => 0x4,
            PacketType::SnakeConnect => 0x5,
            PacketType::SnakeDisconnect => 0x6,
            PacketType::ConnRejected => 0x7,
        };

        let packet_len = (self.buffer.len() + 1) as u16;

        let mut packet = Vec::with_capacity(self.buffer.len() + 3);
        packet.extend_from_slice(&packet_len.to_le_bytes());
        packet.push(packet_type);
        packet.extend_from_slice(&self.buffer);

        packet
    }
}

pub struct ReadablePacket {
    pub r#type: PacketType,
    pub buffer: Vec<u8>,
    cursor: usize,
}

impl ReadablePacket {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let packet_type = match bytes[0] {
            0x1 => PacketType::Info,
            0x2 => PacketType::FoodUpdate,
            0x3 => PacketType::DirectionUpdate,
            0x4 => PacketType::HeadUpdate,
            0x5 => PacketType::SnakeConnect,
            0x6 => PacketType::SnakeDisconnect,
            0x7 => PacketType::ConnRejected,
            _ => panic!("Unknown packet type")
        };

        Self {
            r#type: packet_type,
            buffer: bytes[1..].to_vec(),
            cursor: 0,
        }
    }

    pub fn read(&mut self) -> u8 {
        let val = self.buffer[self.cursor];
        self.cursor += 1;

        val
    }

    pub fn read_u16_le(&mut self) -> u16 {
        let bytes: [u8; 2] = [self.buffer[self.cursor], self.buffer[self.cursor + 1]];

        self.cursor += 2;

        u16::from_le_bytes(bytes)
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.cursor
    }
}
