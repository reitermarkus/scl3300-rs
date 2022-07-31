#![cfg_attr(not(test), no_std)]

use embedded_hal::blocking::{delay::DelayMs, spi::{Write, Transfer}};

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
    let frame = self.transfer(Operation::Reset)?;
    log::debug!("wake_up: {:?}", frame.return_status());
    delay.delay_ms(1);

    let frame = self.transfer(Operation::ChangeMode(mode))?;
    log::debug!("reset: {:?}", frame.return_status());

    let frame = self.transfer(Operation::EnableAngleOutputs)?;
    log::debug!("change_mode: {:?}", frame.return_status());
    delay.delay_ms(mode.wait_time());

    let frame = self.transfer(Operation::Read(Output::Status))?;
    log::debug!("enable_angle_outputs: {:?}", frame.return_status());
    let frame = self.transfer(Operation::Read(Output::Status))?;
    log::debug!("status: {:?}", frame.return_status());
    let frame = self.transfer(Operation::Read(Output::Status))?;
    log::debug!("status: {:?}", frame.return_status());

    frame.check_crc()?;

    Ok(Status::from_bits_truncate(frame.data()))
  }

  pub fn read(&mut self, mut outputs: &mut [(Output, &mut u16)]) -> Result<(), Error<E>> {
    let mut previous_output: Option<&mut u16> = None;

    for (output, value) in outputs {
      let frame = self.transfer(Operation::Read(*output))?;
      frame.check_crc()?;

      if let Some(v) = previous_output.take() {
        *v = frame.data();
      }

      previous_output = Some(value);
    }

    let frame = self.transfer(Operation::Read(Output::Status))?;
    frame.check_crc()?;

    if let Some(v) = previous_output.take() {
      *v = frame.data();
    }

    Ok(())
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

  // pub fn write(&mut self, operation: Operation) -> Result<(), Error<E>> {
//     let mut frame = operation.to_frame();
//
//     log::debug!("write before: {:?}", frame.as_bytes());
//
//     let res = match self.spi.write(frame.as_bytes()) {
//       Ok(()) => Ok(()),
//       Err(err) => Err(Error::Spi(err)),
//     };
//
//     log::debug!("write after: {:?}", frame.as_bytes_mut());
//
//     res
//   }

  fn transfer(&mut self, operation: Operation) -> Result<Frame, Error<E>> {
    let mut frame = operation.to_frame();

    log::debug!("transfer before: {:?}", frame.as_bytes_mut());

    if let Err(err) = self.spi.transfer(frame.as_bytes_mut()) {
      return Err(Error::Spi(err))
    }

    log::debug!("transfer after: {:?}", frame.as_bytes_mut());

    Ok(frame)
  }
}
