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
        log::debug!("Unknown packet: {}", self.op_code);
        Ok(())
    }
}
