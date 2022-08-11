use embedded_hal::blocking::{delay::DelayUs, spi::Transfer};

use crate::{
  Scl3300, Normal,
  Error,
  operation::{Operation, Output, Axis, Bank},
  output::{
    Acceleration, Inclination, Temperature, SelfTest, ComponentId, Serial,
    Status, Error1, Error2,
  },
};

enum OutputVal<'v> {
  U16(&'v mut u16),
  Status(&'v mut Status),
  Error1(&'v mut Error1),
  Error2(&'v mut Error2),
  WhoAmI(&'v mut u8),
}

/// Read measurements and status values.
///
/// This `struct` is returned by the [`read`](Scl3300::read) method on [`Scl3300`](Scl3300).
///
/// The SCL3300 uses an off-frame protocal for reading, meaning the response of a given
/// operation is returned by the next operation. This struct encompasses this fact by
/// combining multiple read operations and sending a single final dummy operation to get
/// the last response when the [`finish`](Reader::finish) method is called.
#[must_use = "`.finish()` must be called to read the last value"]
pub struct Reader<'s, 'd, 'v, SPI, E, D> {
  scl: &'s mut Scl3300<SPI, Normal>,
  delay: &'d mut D,
  previous_value: Option<OutputVal<'v>>,
  bank: Bank,
  error: Result<(), Error<E>>,
}

impl<'s, 'd, 'v, SPI, E, D> Reader<'s, 'd, 'v, SPI, E, D>
where
  SPI: Transfer<u8, Error = E>,
  D: DelayUs<u32>,
{
  #[inline]
  pub(crate) fn new(scl: &'s mut Scl3300<SPI, Normal>, delay: &'d mut D) -> Self {
    Reader { scl, delay, previous_value: None, bank: Bank::Zero, error: Ok(()) }
  }

  fn transfer(&mut self, operation: Operation) -> Result<(), Error<E>> {
    let frame = self.scl.transfer(operation, self.delay, None)?;

    if let Some(v) = self.previous_value.take() {
      let data = frame.data();

      match v {
        OutputVal::U16(v) => {
          *v = data;
        },
        OutputVal::Status(v) => {
          *v = unsafe { Status::from_bits_unchecked(data) };
        },
        OutputVal::Error1(v) => {
          *v = unsafe { Error1::from_bits_unchecked(data) };
        },
        OutputVal::Error2(v) => {
          *v = unsafe { Error2::from_bits_unchecked(data) };
        },
        OutputVal::WhoAmI(v) => {
          *v = data.to_be_bytes()[1];
        }
      }
    }

    Ok(())
  }

  fn switch_to_bank(mut self, bank: Bank) -> Reader<'s, 'd, 'v, SPI, E, D> {
    if self.bank == bank {
      return self
    }

    self.error = if let Err(err) = self.error {
      Err(err)
    } else {
      match self.transfer(Operation::SwitchBank(bank)) {
        Ok(()) => {
          self.bank = bank;
          Ok(())
        },
        Err(err) => Err(err),
      }
    };

    self
  }

  #[inline]
  fn read<'r>(mut self, output: Output, value: Option<OutputVal<'r>>) -> Reader<'s, 'd, 'r, SPI, E, D> {
    let error = if let Err(err) = self.error {
      Err(err)
    } else {
      self.transfer(Operation::Read(output))
    };
    Reader {
      scl: self.scl,
      delay: self.delay,
      previous_value: value,
      bank: self.bank,
      error,
    }
  }

  /// Read the measured acceleration.
  pub fn acceleration<'r>(self, acc: &'r mut Acceleration) -> Reader<'s, 'd, 'r, SPI, E, D> {
    acc.mode = self.scl.mode.mode;
    self.read(Output::Acceleration(Axis::X), Some(OutputVal::U16(&mut acc.x)))
      .read(Output::Acceleration(Axis::Y), Some(OutputVal::U16(&mut acc.y)))
      .read(Output::Acceleration(Axis::Z), Some(OutputVal::U16(&mut acc.z)))
  }

  /// Read the measured inclination.
  pub fn inclination<'r>(self, inc: &'r mut Inclination) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.switch_to_bank(Bank::Zero)
      .read(Output::Angle(Axis::X), Some(OutputVal::U16(&mut inc.x)))
      .read(Output::Angle(Axis::Y), Some(OutputVal::U16(&mut inc.y)))
      .read(Output::Angle(Axis::Z), Some(OutputVal::U16(&mut inc.z)))
  }

  /// Read the measured temperature.
  pub fn temperature<'r>(self, t: &'r mut Temperature) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.read(Output::Temperature, Some(OutputVal::U16(&mut t.temp)))
  }

  /// Read the self-test output.
  pub fn self_test<'r>(self, st: &'r mut SelfTest) -> Reader<'s, 'd, 'r, SPI, E, D> {
    st.mode = self.scl.mode.mode;
    self.read(Output::SelfTest, Some(OutputVal::U16(&mut st.sto)))
  }

  /// Read the `WHOAMI` value.
  pub fn whoami<'r>(self, w: &'r mut ComponentId) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.switch_to_bank(Bank::Zero)
      .read(Output::WhoAmI, Some(OutputVal::WhoAmI(&mut w.id)))
  }

  /// Read the serial number.
  pub fn serial<'r>(self, s: &'r mut Serial) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.switch_to_bank(Bank::One)
      .read(Output::Serial1, Some(OutputVal::U16(&mut s.part1)))
      .read(Output::Serial2, Some(OutputVal::U16(&mut s.part2)))
  }

  /// Read the `STATUS` register.
  pub fn status<'r>(self, s: &'r mut Status) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.switch_to_bank(Bank::Zero)
      .read(Output::Status, Some(OutputVal::Status(s)))
  }

  /// Read the `ERROR1` register.
  pub fn error1<'r>(self, e: &'r mut Error1) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.switch_to_bank(Bank::Zero)
      .read(Output::Error1, Some(OutputVal::Error1(e)))
  }

  /// Read the `ERROR2` register.
  pub fn error2<'r>(self, e: &'r mut Error2) -> Reader<'s, 'd, 'r, SPI, E, D> {
    self.switch_to_bank(Bank::Zero)
      .read(Output::Error2, Some(OutputVal::Error2(e)))
  }

  /// Finish the read transaction.
  pub fn finish(mut self) -> Result<(), Error<E>> {
    if let Err(err) = self.error {
      return Err(err)
    }

    if self.previous_value.is_some() {
      // Ensure `previous_value` is read and bank is reset to 0
      // since bank 1 is only ever needed to read the serial.
      self.transfer(Operation::SwitchBank(Bank::Zero))?;
    }

    Ok(())
  }
}
