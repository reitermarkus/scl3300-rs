//! This is a driver for [SCL3300](https://www.murata.com/en-global/products/sensor/inclinometer/overview/lineup/scl3300)
//! inclinometers, implemented using platform-agnostic [`embedded-hal`](https://docs.rs/embedded-hal/latest/embedded_hal/) traits.
//!
//! # Usage
//!
//! ```rust
//! # fn main() -> Result<(), scl3300::Error<embedded_hal::spi::ErrorKind>> {
//! # use embedded_hal_mock::eh1::{spi::{Mock as SpiMock, Transaction as SpiTransaction}};
//! # let spi = SpiMock::new(&[
//! #   // Reset.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0xB4, 0x00, 0x20, 0x98], vec![3, 0, 0, 125]),
//! #   SpiTransaction::delay(1000000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Change to inclination mode.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0xB4, 0x00, 0x02, 0x25], vec![3, 0, 0, 125]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Enable angle outputs.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0xB0, 0x00, 0x1F, 0x6F], vec![183, 0, 2, 169]),
//! #   SpiTransaction::delay(100000000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read status.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x18, 0x00, 0x00, 0xE5], vec![179, 0, 31, 227]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read status.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x18, 0x00, 0x00, 0xE5], vec![27, 0, 18, 158]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read status.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x18, 0x00, 0x00, 0xE5], vec![25, 0, 18, 157]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read WHOAMI.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x40, 0x00, 0x00, 0x91], vec![25, 0, 0, 106]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Switch to bank 0.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0xFC, 0x00, 0x00, 0x73], vec![65, 0, 193, 54]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read X-axis acceleration.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x04, 0x00, 0x00, 0xF7], vec![25, 0, 0, 106]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read Y-axis acceleration.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x08, 0x00, 0x00, 0xFD], vec![5, 255, 230, 197]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read Z-axis acceleration.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x0C, 0x00, 0x00, 0xFB], vec![9, 0, 141, 213]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read X-axis inclination.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x24, 0x00, 0x00, 0xC7], vec![13, 46, 112, 183]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read Y-axis inclination.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x28, 0x00, 0x00, 0xCD], vec![37, 255, 233, 78]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read Z-axis inclination.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x2C, 0x00, 0x00, 0xCB], vec![41, 0, 123, 212]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Read temperature.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0x14, 0x00, 0x00, 0xEF], vec![45, 63, 129, 29]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Switch to bank 0.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0xFC, 0x00, 0x00, 0x73], vec![21, 22, 20, 216]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! #
//! #   // Power down.
//! #   SpiTransaction::transaction_start(),
//! #   SpiTransaction::transfer_in_place(vec![0xB4, 0x00, 0x04, 0x6B], vec![253, 0, 0, 252]),
//! #   SpiTransaction::delay(10000),
//! #   SpiTransaction::transaction_end(),
//! # ]);
//! use scl3300::{Scl3300, Acceleration, ComponentId, Inclination, MeasurementMode, Temperature};
//!
//! let inclinometer = Scl3300::new(spi);
//!
//! // Start the inclinometer and switch to inclination mode.
//! let mut inclinometer = inclinometer.start_up(MeasurementMode::Inclination)?;
//!
//! // Read the component ID.
//! let id: ComponentId = inclinometer.read()?;
//! assert_eq!(id, ComponentId::WHOAMI);
//!
//! // Read acceleration, inclination and temperature.
//! let (acc, inc, temp): (Acceleration, Inclination, Temperature) = inclinometer.read()?;
//!
//! # assert_eq!(acc.x_g(), -0.0021666666);
//! # assert_eq!(acc.y_g(), 0.01175);
//! # assert_eq!(acc.z_g(), 0.9906667);
//! println!("Acceleration: {}g, {}g, {}g", acc.x_g(), acc.y_g(), acc.z_g());
//! #
//! # assert_eq!(inc.x_degrees(), 359.87366);
//! # assert_eq!(inc.y_degrees(), 0.6756592);
//! # assert_eq!(inc.z_degrees(), 89.30237);
//! println!("Inclination: {}°, {}°, {}°", inc.x_degrees(), inc.y_degrees(), inc.z_degrees());
//! #
//! # assert_eq!(temp.degrees_celsius(), 26.047638);
//! println!("Temperature: {}°C", temp.degrees_celsius());
//!
//! // Switch to power-down mode.
//! let inclinometer = inclinometer.power_down()?;
//!
//! // Release the SPI peripheral again.
//! let spi = inclinometer.release();
//! # let mut spi = spi;
//! # spi.done();
//! drop(spi);
//! # Ok(())
//! # }
//! ```
#![cfg_attr(not(test), no_std)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

use core::{marker::PhantomData, num::NonZeroU32};

use embedded_hal::spi::{Operation as SpiOperation, SpiDevice};

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
mod off_frame_read;
pub use off_frame_read::*;

/// [`Scl3300`](crate::Scl3300) operation modes.
pub mod mode {
  use super::*;

  /// Marker type for an uninitialized [`Scl3300`](crate::Scl3300).
  #[derive(Debug)]
  pub struct Uninitialized {
    pub(crate) _0: PhantomData<()>,
  }

  /// Marker type for a [`Scl3300`](crate::Scl3300) in normal operation mode.
  #[derive(Debug)]
  pub struct Normal {
    pub(crate) mode: MeasurementMode,
  }

  /// Marker type for a [`Scl3300`](crate::Scl3300) in power down mode.
  #[derive(Debug)]
  pub struct PowerDown {
    pub(crate) _0: PhantomData<()>,
  }
}
pub use mode::*;

const MIN_WAIT_TIME_NS: NonZeroU32 = match NonZeroU32::new(10_000) {
  Some(v) => v,
  None => unreachable!(),
};
const WAKE_UP_TIME_NS: NonZeroU32 = match NonZeroU32::new(1_000_000) {
  Some(v) => v,
  None => unreachable!(),
};
const RESET_TIME_NS: NonZeroU32 = match NonZeroU32::new(1_000_000) {
  Some(v) => v,
  None => unreachable!(),
};

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
  SPI: SpiDevice<u8, Error = E>,
{
  /// Start the inclinometer in the given [`MeasurementMode`](enum.MeasurementMode.html).
  fn start_up_inner(mut self, mode: MeasurementMode) -> Result<Scl3300<SPI, Normal>, Error<E>> {
    // Software reset the device.
    self.write(Operation::Reset, Some(RESET_TIME_NS))?;

    // Select operation mode.
    self.write(Operation::ChangeMode(mode), None)?;
    // Enable angle outputs.
    self.write(Operation::EnableAngleOutputs, Some(mode.start_up_wait_time_ns()))?;

    // Clear status summary.
    self.write(Operation::Read(Output::Status), None)?;
    // Read status summary.
    self.write(Operation::Read(Output::Status), None)?;
    // Ensure successful start-up.
    self.transfer(Operation::Read(Output::Status), None)?;

    Ok(Scl3300 { spi: self.spi, mode: Normal { mode } })
  }

  #[inline]
  fn write(&mut self, operation: Operation, wait_us: Option<NonZeroU32>) -> Result<(), Error<E>> {
    self.transfer_inner(operation, wait_us)?;
    Ok(())
  }

  #[inline]
  fn transfer(&mut self, operation: Operation, wait_us: Option<NonZeroU32>) -> Result<Frame, Error<E>> {
    let frame = self.transfer_inner(operation, wait_us)?;
    frame.check_crc()?;

    match frame.return_status() {
      ReturnStatus::StartupInProgress => Err(Error::Startup),
      ReturnStatus::Error => Err(Error::ReturnStatus),
      ReturnStatus::NormalOperation => Ok(frame),
    }
  }

  #[inline]
  fn transfer_inner(&mut self, operation: Operation, wait_us: Option<NonZeroU32>) -> Result<Frame, Error<E>> {
    let mut frame = operation.to_frame();

    let res = self.spi.transaction(&mut [
      SpiOperation::TransferInPlace(frame.as_bytes_mut()),
      SpiOperation::DelayNs(wait_us.unwrap_or(MIN_WAIT_TIME_NS).get()),
    ]);
    if let Err(err) = res {
      return Err(Error::Spi(err))
    }

    Ok(frame)
  }
}

impl<SPI, E> Scl3300<SPI, Uninitialized>
where
  SPI: SpiDevice<u8, Error = E>,
{
  /// Start the inclinometer in the given [`MeasurementMode`](enum.MeasurementMode.html).
  ///
  /// When the inclinometer is in power down mode, use [`wake_up`](Scl3300::wake_up) instead.
  #[inline(always)]
  pub fn start_up(self, mode: MeasurementMode) -> Result<Scl3300<SPI, Normal>, Error<E>> {
    self.start_up_inner(mode)
  }
}

impl<SPI, E> Scl3300<SPI, Normal>
where
  SPI: SpiDevice<u8, Error = E>,
{
  /// Read a value.
  ///
  /// The following outputs are supported:
  ///
  /// - [`Acceleration`](output::Acceleration)
  /// - [`Inclination`](output::Inclination)
  /// - [`Temperature`](output::Temperature)
  /// - [`SelfTest`](output::SelfTest)
  /// - [`ComponentId`](output::ComponentId)
  /// - [`Serial`](output::Serial)
  /// - [`Status`](output::Status)
  /// - [`Error1`](output::Error1)
  /// - [`Error2`](output::Error2)
  ///
  /// Additinally, multiple outputs can be read by specifying a tuple.
  pub fn read<V>(&mut self) -> Result<V, Error<E>>
  where
    V: OffFrameRead<SPI, E>,
  {
    let mut current_bank = Bank::Zero;

    let (_, mut partial) = V::start_read(self, &mut current_bank)?;

    let last_value = self.transfer(Operation::SwitchBank(Bank::Zero), None)?.data();

    partial.finish_read(last_value);

    Ok(partial)
  }

  /// Put the inclinometer into power down mode.
  pub fn power_down(mut self) -> Result<Scl3300<SPI, PowerDown>, Error<E>> {
    self.transfer(Operation::PowerDown, None)?;
    Ok(Scl3300 { spi: self.spi, mode: PowerDown { _0: PhantomData } })
  }
}

impl<SPI, E> Scl3300<SPI, PowerDown>
where
  SPI: SpiDevice<u8, Error = E>,
{
  /// Wake the inclinometer up from power down mode and switch to the given [`MeasurementMode`](enum.MeasurementMode.html).
  #[inline(always)]
  pub fn wake_up(mut self, mode: MeasurementMode) -> Result<Scl3300<SPI, Normal>, Error<E>> {
    self.write(Operation::WakeUp, Some(WAKE_UP_TIME_NS))?;
    self.start_up_inner(mode)
  }
}

impl<SPI, MODE> Scl3300<SPI, MODE> {
  /// Release the contained SPI peripheral.
  pub fn release(self) -> SPI {
    self.spi
  }
}
