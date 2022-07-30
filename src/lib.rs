#![no_std]

use embedded_hal::blocking::{delay::DelayMs, spi::{Write, Transfer}};

mod frame;
pub use frame::*;
mod measurement_mode;
pub use measurement_mode::*;
mod operation;
pub use operation::*;

#[derive(Debug, Clone)]
pub struct Scl3300<SPI> {
  spi: SPI
}

impl<SPI, E> Scl3300<SPI>
where
  SPI: Write<u8, Error = E> + Transfer<u8, Error = E>,
{
  pub fn new(spi: SPI) -> Self {
    Self { spi }
  }

  pub fn measure<D: DelayMs<u8>>(&mut self, delay: &mut D, mode: MeasurementMode) -> Result<(), E> {
    self.write(Operation::WakeUp)?;
    delay.delay_ms(1);

    self.transfer(Operation::Reset)?;
    delay.delay_ms(1);

    self.transfer(Operation::ChangeMode(mode))?;


    delay.delay_ms(mode.wait_time());

    self.transfer(Operation::PowerDown)
  }

  fn write(&mut self, operation: Operation) -> Result<(), E> {
    self.spi.write(&operation.to_frame().as_bytes_mut())
  }

  fn transfer(&mut self, operation: Operation) -> Result<(), E> {
    let mut frame = operation.to_frame();
    self.spi.transfer(frame.as_bytes_mut())?;

    match frame.return_status() {
      ReturnStatus::StartupInProgress => {
        todo!()
      },
      ReturnStatus::NormalOperation => Ok(()),
      ReturnStatus::Error => {
        todo!()
      },
    }
  }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
