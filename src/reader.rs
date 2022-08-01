use embedded_hal::blocking::{delay::DelayUs, spi::Transfer};

use crate::{Scl3300, Acceleration, Inclination, Temperature, SelfTest, WhoAmI, Serial, Operation, Output, Axis, Bank, Error};

#[must_use = "`.finish()` must be called to read the last value"]
pub struct Reader<'s, 'd, 'v, SPI, D> {
  scl: &'s mut Scl3300<SPI>,
  delay: &'d mut D,
  previous_value: Option<&'v mut u16>,
}

impl<'s, 'd, 'v, SPI, E, D> Reader<'s, 'd, 'v, SPI, D>
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  pub(crate) fn new(scl: &'s mut Scl3300<SPI>, delay: &'d mut D) -> Self {
    Reader { scl, delay, previous_value: None }
  }

  fn transfer<'r>(mut self, operation: Operation, value: Option<&'r mut u16>) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    let frame = self.scl.transfer(operation, self.delay, None)?;

    if let Some(v) = self.previous_value.take() {
      *v = frame.data();
    }

    Ok(Reader { scl: self.scl, delay: self.delay, previous_value: value })
  }

  #[inline]
  fn read<'r>(self, output: Output, value: Option<&'r mut u16>) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    self.transfer(Operation::Read(output), value)
  }

  pub fn acceleration<'r>(self, acc: &'r mut Acceleration) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    acc.mode = self.scl.mode.expect("SCL3300 not initialized");
    self.read(Output::Acceleration(Axis::X), Some(&mut acc.x))?
      .read(Output::Acceleration(Axis::Y), Some(&mut acc.y))?
      .read(Output::Acceleration(Axis::Z), Some(&mut acc.z))
  }

  pub fn inclination<'r>(self, inc: &'r mut Inclination) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    self.read(Output::Angle(Axis::X), Some(&mut inc.x))?
      .read(Output::Angle(Axis::Y), Some(&mut inc.y))?
      .read(Output::Angle(Axis::Z), Some(&mut inc.z))
  }

  pub fn temperature<'r>(self, t: &'r mut Temperature) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    self.read(Output::Temperature, Some(&mut t.temp))
  }

  pub fn self_test<'r>(self, st: &'r mut SelfTest) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    st.mode = self.scl.mode.expect("SCL3300 not initialized");
    self.read(Output::Temperature, Some(&mut st.sto))
  }

  pub fn whoami<'r>(self, w: &'r mut WhoAmI) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    self.read(Output::WhoAmI, Some(&mut w.whoami))
  }

  pub fn serial<'r>(self, s: &'r mut Serial) -> Result<Reader<'s, 'd, 'r, SPI, D>, Error<E>> {
    self.transfer(Operation::SwitchBank(Bank::One), None)?
        .read(Output::Serial1, Some(&mut s.part1))?
        .read(Output::Serial2, Some(&mut s.part2))?
        .transfer(Operation::SwitchBank(Bank::Zero), None)
  }

  pub fn finish(self) -> Result<(), Error<E>> {
    let _ = self.read(Output::Status, None)?;
    Ok(())
  }
}
