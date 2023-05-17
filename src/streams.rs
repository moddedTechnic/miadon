pub trait IStream {
    fn read_u8(&mut self) -> u8;

    fn read_u16(&mut self) -> u16 {
        let high = self.read_u8() as u16;
        let low = self.read_u8() as u16;
        (high << 8) | low
    }

    fn read_u32(&mut self) -> u32 {
        let high = self.read_u16() as u32;
        let low  = self.read_u16() as u32;
        (high << 16) | low
    }

    fn read_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.read_u8()).collect::<Vec<_>>()
    }

    fn read_string(&mut self, len: usize) -> String {
        String::from_utf8(self.read_bytes(len)).unwrap()
    }
}


pub trait OStream {
    fn write_u8(&mut self, x: u8);

    fn write_u16(&mut self, x: u16) {
        self.write_u8((x >> 8) as u8);
        self.write_u8((x & 0xff) as u8);
    }

    fn write_u32(&mut self, x: u32) {
        self.write_u16((x >> 16) as u16);
        self.write_u16((x & 0xffff) as u16);
    }

    fn write_bytes(&mut self, bytes: Vec<u8>) {
        bytes.iter().for_each(|c| self.write_u8(c.clone()));
    }

    fn write_string(&mut self, string: String) {
        string.chars().for_each(|c| self.write_u8(c as u8));
    }
}
