use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReturnStatus {
  StartupInProgress,
  NormalOperation,
  Error,
}

#[derive(Debug, Clone)]
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

  pub const fn data(&self) -> u16 {
    u16::from_be_bytes([self.bytes[1], self.bytes[2]])
  }

  /// Compare the CRC of the input array to the given CRC checksum.
  pub fn check_crc<E>(&self) -> Result<(), Error<E>> {
    let crc = self.bytes[3];
    let calculated_crc = crc8([self.bytes[0], self.bytes[1], self.bytes[2]]);

    if calculated_crc == crc {
      Ok(())
    } else {
      Err(Error::Crc)
    }
  }

  pub fn as_bytes(&self) -> &[u8] {
    &self.bytes
  }

  pub fn as_bytes_mut(&mut self) -> &mut [u8] {
    &mut self.bytes
  }
}

/// Calculate the CRC8 checksum for the given input array.
fn crc8(data: [u8; 3]) -> u8 {
  let mut crc = 0xff;

  for byte in data {
    crc ^= byte;

    for _ in 0..8 {
      if crc & 0x80 > 0 {
        crc = (crc << 1) ^ 0x1d;
      } else {
        crc <<= 1;
      }
    }
  }

  !crc
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_crc8() {
    let examples = [
      [183, 0, 2, 169],
      [25, 0, 18, 157],
      [25, 0, 0, 106],
      [27, 0, 18, 158],
      [24, 0, 0, 229],
      [183, 0, 0, 147],
    ];

    for example in examples {
      let data = [example[0], example[1], example[2]];
      let crc = example[3];

      assert_eq!(crc8(data), crc, "CRC check failed for {:?}", example);
    }
  }
}
