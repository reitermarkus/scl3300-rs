pub enum ReturnStatus {
  StartupInProgress,
  NormalOperation,
  Error,
}

pub struct Frame {
  pub(crate) bytes: [u8; 4],
}

impl Frame {
  pub const fn return_status(&self) -> ReturnStatus {
    use ReturnStatus::*;

    match self.bytes[0] & 0b11 {
      0b00 => StartupInProgress,
      0b01 => NormalOperation,
      0b11 => Error,
      _ => unreachable!(),
    }
  }

  pub fn as_bytes_mut(&mut self) -> &mut [u8] {
    &mut self.bytes
  }
}
