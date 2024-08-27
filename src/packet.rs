type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub struct PacketBuffer{
    pub buf: [u8; 512],
    pub pos: usize
}

impl PacketBuffer{
    pub fn new() -> PacketBuffer {
        PacketBuffer{
            buf: [0;512],
            pos: 0
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps:usize) -> Result<()> {
        self.pos += steps;

        Ok(())
    }

    pub fn seek(&mut self, pos:usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos >= 512{
            return Err("Error: Tried to read past the end of the buffer".into());
        }
        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    pub fn get(&mut self, pos:usize) -> Result<u8> {
        if self.pos >= 512{
            return Err("Error: Tried to get location outside buffer".into());
        }
        Ok(self.buf[pos])
    }

    pub fn get_range(&mut self, start: usize, len:usize) ->Result<&[u8]>{
        if self.pos >= 512{
            return Err("Error: Tried to get range outside buffer".into());
        }

        Ok(&self.buf[start..(start + len as usize )])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let res =  ((self.read()? as u16) << 8) | (self.read()? as u16);

        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32>{
        let res = ((self.read()? as u32) << 24)
                | ((self.read()? as u32) << 16)
                | ((self.read()? as u32) << 8)
                | ((self.read()? as u32) << 0);

        Ok(res)
    }

    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()>{
        const MAX_JUMPS:i32 = 5;

        let mut pos = self.pos();
        let mut jumped = false;

        let mut delim = "";
        let mut num_jumps = 0;

        loop{
            if num_jumps > MAX_JUMPS {
                return Err(("Exceeded maximum number of jumps allowed. There might be a cycle in the jump instructions").into())
            }

            let length = self.get(pos)?;

            if (length & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos+2)?;
                }

                let bytes = self.get(pos +1)? as u16;
                let offset = (((length as u16) ^ 0xC0) << 8) | bytes;
                pos = offset  as usize;
                jumped = true;
                num_jumps += 1;
                continue;
            }

            pos+= 1;

            if length == 0{break}

            outstr.push_str(delim);

            let str_buffer = self.get_range(pos, length as usize)?;
            outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

            delim = ".";

            pos += length as usize;
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

    pub fn write(&mut self, val:u8) -> Result<()> {
        if self.pos >= 512{
            return Err("Trying to write past end of buffer".into());
        }

        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<()>{
        self.write(val)?;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<()>{
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x34 {
                return Err("Error label exceeds 63 characters".into());
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }

        self.write_u8(0)?;

        Ok(())
    }

    pub fn set(&mut self, pos: usize, val: u8) -> Result<()> {
        self.buf[pos] = val;

        Ok(())
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) -> Result<()> {
        self.set(pos, (val >> 8) as u8)?;
        self.set(pos + 1, (val & 0xFF) as u8)?;

        Ok(())
    }
}