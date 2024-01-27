pub struct Packet {
    pub buffer: Vec<u8>,
}

impl Packet {
    pub fn new() -> Self {
        Self { buffer: vec![0, 0] }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity + 2);

        buffer.push(0);
        buffer.push(0);

        Self { buffer }
    }

    pub fn write(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn write_u16_le(&mut self, value: u16) {
        let b0 = (value & 0xff) as u8;
        let b1 = ((value >> 8) & 0xff) as u8;

        self.buffer.push(b0);
        self.buffer.push(b1);
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
