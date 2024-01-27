pub struct Packet {
    pub buffer: Vec<u8>,
    offset: usize,
}

impl Packet {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity + 2);

        buffer.push(0);
        buffer.push(0);

        Self { buffer, offset: 2 }
    }

    pub fn from(bytes: Vec<u8>) -> Self {
        Self {
            buffer: bytes,
            offset: 0,
        }
    }

    pub fn read(&mut self) -> u8 {
        let val = self.buffer[self.offset];
        self.offset += 1;

        val
    }

    pub fn read_u16_le(&mut self) -> u16 {
        let b0 = self.buffer[self.offset] as u16;
        let b1 = self.buffer[self.offset + 1] as u16;
        self.offset += 2;

        b0 | (b1 << 8)
    }

    pub fn write(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.offset
    }

    pub fn build(&mut self) -> &[u8] {
        let len = self.buffer.len() - 2;
        let b0 = (len & 0xff) as u8;
        let b1 = ((len >> 8) & 0xff) as u8;

        self.buffer[0] = b0;
        self.buffer[1] = b1;

        &self.buffer
    }
}