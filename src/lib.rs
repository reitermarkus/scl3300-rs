#![cfg_attr(not(test), no_std)]

use embedded_hal::blocking::{delay::DelayMs, spi::{Transfer}};

mod error;
pub use error::*;
mod frame;
pub use frame::*;
mod measurement_mode;
pub use measurement_mode::*;
mod operation;
pub use operation::*;
mod status;
pub use status::*;

#[derive(Debug, Clone)]
pub struct Scl3300<SPI> {
  spi: SPI,
}

impl<SPI, E> Scl3300<SPI>
where
  SPI: Transfer<u8, Error = E>,
{
  pub const fn new(spi: SPI) -> Self {
    Self { spi }
  }

  pub fn wake_up<D: DelayMs<u8>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
    self.transfer(Operation::WakeUp)?;
    delay.delay_ms(1);

    Ok(())
  }

  pub fn init<D: DelayMs<u8>>(&mut self, delay: &mut D, mode: MeasurementMode) -> Result<Status, Error<E>> {
    self.transfer(Operation::Reset)?;
    delay.delay_ms(1);

    self.transfer(Operation::ChangeMode(mode))?;
    self.transfer(Operation::EnableAngleOutputs)?;
    delay.delay_ms(mode.wait_time());

    self.transfer(Operation::Read(Output::Status))?;
    self.transfer(Operation::Read(Output::Status))?;
    
    let frame = self.transfer(Operation::Read(Output::Status))?;
    frame.check_crc()?;
    Ok(Status::from_bits_truncate(frame.data()))
  }
  
  pub fn read(&mut self) -> Reader<'_, '_, SPI> {
    Reader {
      scl: self,
      value: None,
    }
  }
  
  pub fn whoami(&mut self) -> Result<bool, Error<E>> {
    self.transfer(Operation::Read(Output::WhoAmI))?;
    let frame = self.transfer(Operation::Read(Output::Status))?;

    frame.check_crc()?;

    Ok(frame.data() == 0xC1)
  }

  pub fn power_down(&mut self) -> Result<(), Error<E>> {
    self.transfer(Operation::PowerDown)?;

    Ok(())
  }

  fn transfer(&mut self, operation: Operation) -> Result<Frame, Error<E>> {
    let mut frame = operation.to_frame();

    if let Err(err) = self.spi.transfer(frame.as_bytes_mut()) {
      return Err(Error::Spi(err))
    }

    Ok(frame)
  }
}

#[must_use = "`.finish()` must be called to read the last value"]
pub struct Reader<'s, 'v, SPI> {
  scl: &'s mut Scl3300<SPI>,
  value: Option<&'v mut u16>,
}

impl<'s, 'v, SPI, E> Reader<'s, 'v, SPI> 
where
  SPI: Transfer<u8, Error = E>,
{
  fn read<'r>(mut self, output: Output, value: Option<&'r mut u16>) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    let frame = self.scl.transfer(Operation::Read(output))?;
    frame.check_crc()?;

    if let Some(v) = self.value.take() {
      *v = frame.data();
    }

    Ok(Reader {
      scl: self.scl,
      value,
    })
  }
  
  pub fn acceleration_x<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Acceleration(Axis::X), Some(v))
  }
  
  pub fn acceleration_y<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Acceleration(Axis::Y), Some(v))
  }
  
  pub fn acceleration_z<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Acceleration(Axis::Z), Some(v))
  }

  pub fn angle_x<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Angle(Axis::X), Some(v))
  }
  
  pub fn angle_y<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Angle(Axis::Y), Some(v))
  }
  
  pub fn angle_z<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Angle(Axis::Z), Some(v))
  }
  
  pub fn temperature<'r>(self, v: &'r mut u16) -> Result<Reader<'s, 'r, SPI>, Error<E>> {
    self.read(Output::Temperature, Some(v))
  }

  pub fn finish(self) -> Result<(), Error<E>> {
    let _ = self.read(Output::Status, None)?;
    Ok(())
  }
}
