use crate::Result;

#[derive(Debug)]
pub struct Unknown {
    op_code: i16,
}

impl Unknown {
    pub fn new(op_code: i16) -> Self {
        Self { op_code }
    }

    pub fn handle(self) -> Result<()> {
        log::debug!("Unknown packet: 0x{:02x}", self.op_code);
        Ok(())
    }
}
