//! This is a driver for [SCL3300](https://www.murata.com/en-global/products/sensor/inclinometer/overview/lineup/scl3300)
//! inclinometers, implemented using platform-agnostic [`embedded-hal`](https://docs.rs/embedded-hal/latest/embedded_hal/) traits.
//!
//! # Usage
//!
//! ```rust
//! # fn main() -> Result<(), scl3300::Error<embedded_hal_mock::MockError>> {
//! # use embedded_hal_mock::{spi::{Mock as SpiMock, Transaction as SpiTransaction}, delay::MockNoop as Delay};
//! # let spi = SpiMock::new(&[
//! #   SpiTransaction::transfer(vec![0xB4, 0x00, 0x20, 0x98], vec![3, 0, 0, 125]), // Reset.
//! #   SpiTransaction::transfer(vec![0xB4, 0x00, 0x02, 0x25], vec![3, 0, 0, 125]), // Change to inclination mode.
//! #   SpiTransaction::transfer(vec![0xB0, 0x00, 0x1F, 0x6F], vec![183, 0, 2, 169]), // Enable angle outputs.
//! #   SpiTransaction::transfer(vec![0x18, 0x00, 0x00, 0xE5], vec![179, 0, 31, 227]), // Read status.
//! #   SpiTransaction::transfer(vec![0x18, 0x00, 0x00, 0xE5], vec![27, 0, 18, 158]), // Read status.
//! #   SpiTransaction::transfer(vec![0x18, 0x00, 0x00, 0xE5], vec![25, 0, 18, 157]), // Read status.
//! #
//! #   SpiTransaction::transfer(vec![0x40, 0x00, 0x00, 0x91], vec![25, 0, 0, 106]), // Read WHOAMI.
//! #   SpiTransaction::transfer(vec![0xFC, 0x00, 0x00, 0x73], vec![65, 0, 193, 54]), // Switch to bank 0.
//! #
//! #   SpiTransaction::transfer(vec![0x04, 0x00, 0x00, 0xF7], vec![25, 0, 0, 106]), // Read X-axis acceleration.
//! #   SpiTransaction::transfer(vec![0x08, 0x00, 0x00, 0xFD], vec![5, 255, 230, 197]), // Read Y-axis acceleration.
//! #   SpiTransaction::transfer(vec![0x0C, 0x00, 0x00, 0xFB], vec![9, 0, 141, 213]), // Read Z-axis acceleration.
//! #
//! #   SpiTransaction::transfer(vec![0x24, 0x00, 0x00, 0xC7], vec![13, 46, 112, 183]), // Read X-axis inclination.
//! #   SpiTransaction::transfer(vec![0x28, 0x00, 0x00, 0xCD], vec![37, 255, 233, 78]), // Read Y-axis inclination.
//! #   SpiTransaction::transfer(vec![0x2C, 0x00, 0x00, 0xCB], vec![41, 0, 123, 212]), // Read Z-axis inclination.
//! #
//! #   SpiTransaction::transfer(vec![0x14, 0x00, 0x00, 0xEF], vec![45, 63, 129, 29]), // Read temperature.
//! #   SpiTransaction::transfer(vec![0xFC, 0x00, 0x00, 0x73], vec![21, 22, 20, 216]), // Switch to bank 0.
//! #
//! #   SpiTransaction::transfer(vec![0xB4, 0x00, 0x04, 0x6B], vec![253, 0, 0, 252]), // Power down.
//! # ]);
//! # let mut delay = Delay;
//! use scl3300::{Scl3300, Acceleration, ComponentId, Inclination, MeasurementMode, Temperature};
//!
//! let inclinometer = Scl3300::new(spi);
//!
//! // Start the inclinometer and switch to inclination mode.
//! let mut inclinometer = inclinometer.start_up(&mut delay, MeasurementMode::Inclination)?;
//!
//! // Read the component ID.
//! let mut id = ComponentId::new();
//! inclinometer.read(&mut delay)
//!   .component_id(&mut id)
//!   .finish()?;
//! assert!(id.is_correct());
//!
//! // Read acceleration, inclination and temperature.
//! let mut acc = Acceleration::new();
//! let mut inc = Inclination::new();
//! let mut temp = Temperature::new();
//! inclinometer.read(&mut delay)
//!   .acceleration(&mut acc)
//!   .inclination(&mut inc)
//!   .temperature(&mut temp)
//!   .finish()?;
//!
//! # assert_eq!(acc.x_g(), -0.0021666666);
//! # assert_eq!(acc.y_g(), 0.01175);
//! # assert_eq!(acc.z_g(), 0.9906667);
//! println!("Acceleration: {}g, {}g, {}g", acc.x_g(), acc.y_g(), acc.z_g());
//! #
//! # assert_eq!(inc.x_degrees(), 359.87366);
//! # assert_eq!(inc.y_degrees(), 0.6756592);
//! # assert_eq!(inc.z_degrees(), 89.30237);
//! println!("Inclination: {}??, {}??, {}??", inc.x_degrees(), inc.y_degrees(), inc.z_degrees());
//! #
//! # assert_eq!(temp.degrees_celsius(), 26.047638);
//! println!("Temperature: {}??C", temp.degrees_celsius());
//!
//! // Switch to power-down mode.
//! let inclinometer = inclinometer.power_down(&mut delay)?;
//!
//! // Release the SPI peripheral again.
//! let spi = inclinometer.release();
//! # let mut spi = spi;
//! # spi.done();
//! drop(spi);
//! # Ok(())
//! # }
//! ```
#![no_std]

#[cfg(test)]
extern crate alloc;

use core::{num::NonZeroU32, marker::PhantomData};

use embedded_hal::blocking::{delay::DelayUs, spi::{Transfer}};

mod error;
pub use error::*;
mod frame;
use frame::*;
pub mod output;
pub use output::*;
mod measurement_mode;
pub use measurement_mode::*;
mod operation;
use operation::*;
mod reader;
pub use reader::*;


pub mod mode {
  use super::*;

  /// Marks an uninitialized [`Scl3300`](../struct.Scl3300.html).
  #[derive(Debug)]
  pub struct Uninitialized {
    pub(crate) _0: PhantomData<()>,
  }

  /// Marks a [`Scl3300`](../struct.Scl3300.html) in normal operation mode.
  #[derive(Debug)]
  pub struct Normal {
    pub(crate) mode: MeasurementMode,
  }

  /// Marks a [`Scl3300`](../struct.Scl3300.html) in power down mode.
  #[derive(Debug)]
  pub struct PowerDown {
    pub(crate) _0: PhantomData<()>,
  }
}
pub use mode::*;

// SAFTEY: 10 is not 0.
const MIN_WAIT_TIME_US: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(10) };
// SAFTEY: 1000 is not 0.
const WAKE_UP_TIME_US: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1000) };
// SAFTEY: 1000 is not 0.
const RESET_TIME_US: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1000) };

/// An SCL3300 inclinometer.
#[derive(Debug, Clone)]
pub struct Scl3300<SPI, MODE = Uninitialized> {
  pub(crate) spi: SPI,
  pub(crate) mode: MODE,
}

impl<SPI> Scl3300<SPI> {
  /// Create a new `Scl3300` with the given `SPI` instance.
  pub const fn new(spi: SPI) -> Self {
    Scl3300 { spi, mode: Uninitialized { _0: PhantomData } }
  }
}

impl<SPI, E, MODE> Scl3300<SPI, MODE>
where
  SPI: Transfer<u8, Error = E>,
{
  /// Start the inclinometer in the given [`MeasurementMode`](enum.MeasurementMode.html).
  fn start_up_inner<D: DelayUs<u32>>(mut self, delay: &mut D, mode: MeasurementMode) -> Result<Scl3300<SPI, Normal>, Error<E>> {
    // Software reset the device.
    self.write(Operation::Reset, delay, Some(RESET_TIME_US))?;

    // Select operation mode.
    self.write(Operation::ChangeMode(mode), delay, None)?;
    // Enable angle outputs.
    self.write(Operation::EnableAngleOutputs, delay, Some(mode.start_up_wait_time_us()))?;

    // Clear status summary.
    self.write(Operation::Read(Output::Status), delay, None)?;
    // Read status summary.
    self.write(Operation::Read(Output::Status), delay, None)?;
    // Ensure successful start-up.
    self.transfer(Operation::Read(Output::Status), delay, None)?;

    Ok(Scl3300 { spi: self.spi, mode: Normal { mode } })
  }

  #[inline]
  fn write<D: DelayUs<u32>>(&mut self, operation: Operation, delay: &mut D, wait_us: Option<NonZeroU32>) -> Result<(), Error<E>> {
    self.transfer_inner(operation, delay, wait_us)?;
    Ok(())
  }

  #[inline]
  fn transfer<D: DelayUs<u32>>(&mut self, operation: Operation, delay: &mut D, wait_us: Option<NonZeroU32>) -> Result<Frame, Error<E>> {
    let frame = self.transfer_inner(operation, delay, wait_us)?;
    frame.check_crc()?;

    match frame.return_status() {
      ReturnStatus::StartupInProgress => Err(Error::Startup),
      ReturnStatus::Error => Err(Error::ReturnStatus),
      ReturnStatus::NormalOperation => Ok(frame)
    }
  }

  #[inline]
  fn transfer_inner<D: DelayUs<u32>>(&mut self, operation: Operation, delay: &mut D, wait_us: Option<NonZeroU32>) -> Result<Frame, Error<E>> {
    let mut frame = operation.to_frame();
    let res = self.spi.transfer(frame.as_bytes_mut());
    delay.delay_us(wait_us.unwrap_or(MIN_WAIT_TIME_US).get());
    if let Err(err) = res {
      return Err(Error::Spi(err))
    }

    Ok(frame)
  }
}

impl<SPI, E> Scl3300<SPI, Uninitialized>
where
  SPI: Transfer<u8, Error = E>,
{
  /// Start the inclinometer in the given [`MeasurementMode`](enum.MeasurementMode.html).
  ///
  /// When the inclinometer is in power down mode, use [`wake_up`](Scl3300::wake_up) instead.
  #[inline(always)]
  pub fn start_up<D: DelayUs<u32>>(self, delay: &mut D, mode: MeasurementMode) -> Result<Scl3300<SPI, Normal>, Error<E>> {
    self.start_up_inner(delay, mode)
  }
}

impl<SPI, E> Scl3300<SPI, Normal>
where
  SPI: Transfer<u8, Error = E>,
{
  /// Start a read transaction, see [`Reader`](struct.Reader.html) for more information.
  pub fn read<'d, D: DelayUs<u32>>(&mut self, delay: &'d mut D) -> Reader<'_, 'd, 'static, SPI, E, D> {
    Reader::new(self, delay)
  }

  /// Put the inclinometer into power down mode.
  pub fn power_down<D: DelayUs<u32>>(mut self, delay: &mut D) -> Result<Scl3300<SPI, PowerDown>, Error<E>> {
    self.transfer(Operation::PowerDown, delay, None)?;
    Ok(Scl3300 { spi: self.spi, mode: PowerDown { _0: PhantomData } })
  }
}

impl<SPI, E> Scl3300<SPI, PowerDown>
where
  SPI: Transfer<u8, Error = E>,
{
  /// Wake the inclinometer up from power down mode and switch to the given [`MeasurementMode`](enum.MeasurementMode.html).
  #[inline(always)]
  pub fn wake_up<D: DelayUs<u32>>(mut self, delay: &mut D, mode: MeasurementMode) -> Result<Scl3300<SPI, Normal>, Error<E>> {
    self.write(Operation::WakeUp, delay, Some(WAKE_UP_TIME_US))?;
    self.start_up_inner(delay, mode)
  }
}

impl<SPI, MODE> Scl3300<SPI, MODE> {
  /// Release the contained SPI peripheral.
  pub fn release(self) -> SPI {
    self.spi
  }
}
