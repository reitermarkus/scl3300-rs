use embedded_hal::blocking::{delay::DelayUs, spi::Transfer};

use crate::{
  operation::{Bank, Operation, Output},
  output::{Acceleration, ComponentId, Error1, Error2, Inclination, SelfTest, Serial, Status, Temperature},
  Error, Normal, Scl3300,
};

fn transfer_with_bank<SPI, D, E>(
  scl: &mut Scl3300<SPI, Normal>,
  delay: &mut D,
  current_bank: &mut Bank,
  required_bank: Bank,
  operation: Operation,
) -> Result<u16, Error<E>>
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  let mut last_value1 = None;

  if *current_bank != required_bank {
    last_value1 = Some(scl.transfer(Operation::SwitchBank(required_bank), delay, None)?.data());
    *current_bank = required_bank;
  }

  let last_value2 = scl.transfer(operation, delay, None)?.data();

  Ok(last_value1.unwrap_or(last_value2))
}

/// Types implementing this trait can be read using [`Scl3300::read`](crate::Scl3300::read).
pub trait OffFrameRead<SPI, D, E>: Sized
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  /// Start an off-frame read.
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>>;

  /// Finish an off-frame read.
  fn finish_read(&mut self, last_value: u16);
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Acceleration
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    _current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let mut acc = Acceleration { x: 0, y: 0, z: 0, mode: scl.mode.mode };

    let last_value = scl.transfer(Operation::Read(Output::AccelerationX), delay, None)?.data();
    acc.x = scl.transfer(Operation::Read(Output::AccelerationY), delay, None)?.data();
    acc.y = scl.transfer(Operation::Read(Output::AccelerationZ), delay, None)?.data();
    Ok((last_value, acc))
  }

  fn finish_read(&mut self, last_value: u16) {
    self.z = last_value;
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Inclination
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let mut inc = Inclination { x: 0, y: 0, z: 0 };
    let last_value = transfer_with_bank(scl, delay, current_bank, Bank::Zero, Operation::Read(Output::AngleX))?;
    inc.x = scl.transfer(Operation::Read(Output::AngleY), delay, None)?.data();
    inc.y = scl.transfer(Operation::Read(Output::AngleZ), delay, None)?.data();
    Ok((last_value, inc))
  }

  fn finish_read(&mut self, last_value: u16) {
    self.z = last_value;
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Temperature
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    _current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let temp = Temperature { temp: 0 };
    let last_value = scl.transfer(Operation::Read(Output::Temperature), delay, None)?.data();
    Ok((last_value, temp))
  }

  fn finish_read(&mut self, last_value: u16) {
    self.temp = last_value;
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for SelfTest
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    _current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let st = SelfTest { sto: 0, mode: scl.mode.mode };
    let last_value = scl.transfer(Operation::Read(Output::SelfTest), delay, None)?.data();
    Ok((last_value, st))
  }

  fn finish_read(&mut self, last_value: u16) {
    self.sto = last_value;
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for ComponentId
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let id = ComponentId { id: 0 };
    let last_value = transfer_with_bank(scl, delay, current_bank, Bank::Zero, Operation::Read(Output::WhoAmI))?;
    Ok((last_value, id))
  }

  fn finish_read(&mut self, last_value: u16) {
    self.id = last_value.to_be_bytes()[1];
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Serial
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let mut serial = Serial { part1: 0, part2: 0 };
    let last_value = transfer_with_bank(scl, delay, current_bank, Bank::One, Operation::Read(Output::Serial1))?;
    serial.part1 = scl.transfer(Operation::Read(Output::Serial2), delay, None)?.data();
    Ok((last_value, serial))
  }

  fn finish_read(&mut self, last_value: u16) {
    self.part2 = last_value;
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Status
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let status = Self::from_bits_retain(0);
    let last_value = transfer_with_bank(scl, delay, current_bank, Bank::Zero, Operation::Read(Output::Status))?;
    Ok((last_value, status))
  }

  fn finish_read(&mut self, last_value: u16) {
    *self = Self::from_bits_retain(last_value)
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Error1
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let status = Self::from_bits_retain(0);
    let last_value = transfer_with_bank(scl, delay, current_bank, Bank::Zero, Operation::Read(Output::Error1))?;
    Ok((last_value, status))
  }

  fn finish_read(&mut self, last_value: u16) {
    *self = Self::from_bits_retain(last_value)
  }
}

impl<SPI, D, E> OffFrameRead<SPI, D, E> for Error2
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  fn start_read(
    scl: &mut Scl3300<SPI, Normal>,
    delay: &mut D,
    current_bank: &mut Bank,
  ) -> Result<(u16, Self), Error<E>> {
    let status = Self::from_bits_retain(0);
    let last_value = transfer_with_bank(scl, delay, current_bank, Bank::Zero, Operation::Read(Output::Error2))?;
    Ok((last_value, status))
  }

  fn finish_read(&mut self, last_value: u16) {
    *self = Self::from_bits_retain(last_value)
  }
}

macro_rules! off_frame_read_tuple {
  ($($var:ident: $value:ident),+) => {
    impl<SPI, D, E, $($value),+> OffFrameRead<SPI, D, E> for ($($value),+)
    where
      SPI: Transfer<u8, Error = E>,
      D: DelayUs<u32>,
      $(
        $value: OffFrameRead<SPI, D, E>,
      )+
    {
      fn start_read(scl: &mut Scl3300<SPI, Normal>, delay: &mut D, current_bank: &mut Bank) -> Result<(u16, Self), Error<E>> {
        off_frame_read_tuple!(@start_read scl, delay, current_bank, last_value, $($var: $value),+);
        Ok((last_value, ($($var),+)))
      }

      off_frame_read_tuple!(@finish $($var),+);
    }
  };
  (@finish $first_var:ident, $($var:ident),+) => {
    fn finish_read(&mut self, last_value: u16) {
      let ($(off_frame_read_tuple!(@_ $var)),+, last) = self;
      last.finish_read(last_value);
    }
  };
  (@_ $id:ident) => { _ };
  (@start_read
    $scl:expr, $delay:expr, $current_bank:expr,
    $last_value:ident,
    $current_var:ident: $current_value:ident,
    $($var:ident: $value:ident),+
  ) => {
    let ($last_value, mut $current_var) = <$current_value>::start_read($scl, $delay, $current_bank)?;
    off_frame_read_tuple!(@start_read_inner $scl, $delay, $current_bank, $current_var: $current_value, $($var: $value),+);
  };
  (@start_read_inner
    $scl:expr, $delay:expr, $current_bank:expr,
    $previous_var:ident: $previous_value:ident,
    $current_var:ident: $current_value:ident
  ) => {
    let (last_value, $current_var) = <$current_value>::start_read($scl, $delay, $current_bank)?;
    $previous_var.finish_read(last_value);
  };
  (@start_read_inner
    $scl:expr, $delay:expr, $current_bank:expr,
    $previous_var:ident: $previous_value:ident,
    $current_var:ident: $current_value:ident,
    $($var:ident: $value:ident),+
  ) => {
    let (last_value, mut $current_var) = <$current_value>::start_read($scl, $delay, $current_bank)?;
    $previous_var.finish_read(last_value);
    off_frame_read_tuple!(@start_read_inner $scl, $delay, $current_bank, $current_var: $current_value, $($var: $value),+);
  };
}

off_frame_read_tuple!(v1: V1, v2: V2);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4, v5: V5);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4, v5: V5, v6: V6);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4, v5: V5, v6: V6, v7: V7);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4, v5: V5, v6: V6, v7: V7, v8: V8);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4, v5: V5, v6: V6, v7: V7, v8: V8, v9: V9);
off_frame_read_tuple!(v1: V1, v2: V2, v3: V3, v4: V4, v5: V5, v6: V6, v7: V7, v8: V8, v9: V9, v10: V10);
