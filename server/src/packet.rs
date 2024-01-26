pub struct Packet {
    pub buffer: Vec<u8>,
}

impl Packet {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn write(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn write_u16_le(&mut self, value: u16) {
        let b0 = (value & 0xff) as u8;
        let b1 = ((value >> 2) & 0xff) as u8;

        self.buffer.push(b0);
        self.buffer.push(b1);
    }
}
