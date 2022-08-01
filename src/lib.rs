#![cfg_attr(not(test), no_std)]

use core::num::NonZeroU32;

use embedded_hal::blocking::{delay::DelayUs, spi::{Transfer}};

mod error;
pub use error::*;
mod frame;
pub use frame::*;
mod measurement;
pub use measurement::*;
mod measurement_mode;
pub use measurement_mode::*;
mod operation;
use operation::*;
mod reader;
pub use reader::*;
mod status;
pub use status::*;

/// Uninitialized
pub struct Uninitialized;

/// Normal operation mode
pub struct Normal;

/// Power down mode
pub struct PowerDown;

// SAFTEY: 10 is not 0.
const MIN_WAIT_TIME_US: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(10) };
// SAFTEY: 1000 is not 0.
const WAKE_UP_TIME_US: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1000) };
// SAFTEY: 1000 is not 0.
const RESET_TIME_US: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1000) };

#[derive(Debug, Clone)]
pub struct Scl3300<SPI> {
  pub(crate) spi: SPI,
  pub(crate) mode: Option<MeasurementMode>,
  power_down_mode: bool,
}

impl<SPI, E> Scl3300<SPI>
where
  SPI: Transfer<u8, Error = E>,
{
  pub const fn new(spi: SPI) -> Self {
    Self { spi, mode: None, power_down_mode: false }
  }

  /// Start the inclinometer in the given `mode`.
  pub fn start_up<D: DelayUs<u32>>(&mut self, delay: &mut D, mode: MeasurementMode) -> Result<Status, Error<E>> {
    // If mode was previously set, we ware in power down mode.
    if self.power_down_mode {
      self.wake_up(delay)?;
    }

    self.write(Operation::Reset, delay, Some(RESET_TIME_US))?;

    self.write(Operation::ChangeMode(mode), delay, None)?;
    self.mode = Some(mode);
    self.write(Operation::EnableAngleOutputs, delay, Some(mode.start_up_wait_time_us()))?;

    self.write(Operation::Read(Output::Status), delay, None)?;
    self.write(Operation::Read(Output::Status), delay, None)?;

    let frame = self.transfer(Operation::Read(Output::Status), delay, None)?;
    Ok(unsafe { Status::from_bits_unchecked(frame.data()) })
  }

  /// Start a read transaction, see [`Reader`](struct.Reader.html) for more information.
  pub fn read<'d, D: DelayUs<u32>>(&mut self, delay: &'d mut D) -> Reader<'_, 'd, 'static, SPI, E, D> {
    Reader::new(self, delay)
  }

  /// Put the inclinometer into power down mode.
  pub fn power_down<D: DelayUs<u32>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
    self.transfer(Operation::PowerDown, delay, None)?;
    self.power_down_mode = true;
    Ok(())
  }

  /// Wake the inclinometer up from power down mode.
  pub fn wake_up<D: DelayUs<u32>>(&mut self, delay: &mut D) -> Result<(), Error<E>> {
    self.write(Operation::WakeUp, delay, Some(WAKE_UP_TIME_US))?;
    Ok(())
  }

  fn write<D: DelayUs<u32>>(&mut self, operation: Operation, delay: &mut D, wait_us: Option<NonZeroU32>) -> Result<(), Error<E>> {
    self.transfer_inner(operation, delay, wait_us)?;
    Ok(())
  }

  fn transfer<D: DelayUs<u32>>(&mut self, operation: Operation, delay: &mut D, wait_us: Option<NonZeroU32>) -> Result<Frame, Error<E>> {
    let frame = self.transfer_inner(operation, delay, wait_us)?;
    frame.check_crc()?;

    match frame.return_status() {
      ReturnStatus::StartupInProgress => Err(Error::Startup),
      ReturnStatus::Error => Err(Error::ReturnStatus),
      ReturnStatus::NormalOperation => Ok(frame)
    }
  }

  fn transfer_inner<D: DelayUs<u32>>(&mut self, operation: Operation, delay: &mut D, wait_us: Option<NonZeroU32>) -> Result<Frame, Error<E>> {
    let mut frame = operation.to_frame();
    let res = self.spi.transfer(frame.as_bytes_mut());
    delay.delay_us(wait_us.unwrap_or(MIN_WAIT_TIME_US).get());
    if let Err(err) = res {
      return Err(Error::Spi(err))
    }

    Ok(frame)
  }

  /// Release the contained SPI peripheral.
  pub fn release(self) -> SPI {
    self.spi
  }
}
