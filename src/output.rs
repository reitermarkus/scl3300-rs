//! This module includes all types which can be read using [`Scl3300::read`](crate::Scl3300::read).

use core::fmt;

use bitflags::bitflags;

use crate::MeasurementMode;

/// An acceleration measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct Acceleration {
  pub(crate) x: u16,
  pub(crate) y: u16,
  pub(crate) z: u16,
  pub(crate) mode: MeasurementMode,
}

impl Acceleration {
  /// Get the raw acceleration value in the X-direction.
  #[inline(always)]
  pub fn x_raw(&self) -> u16 {
    self.x
  }

  /// Get the raw acceleration value in the Y-direction.
  #[inline(always)]
  pub fn y_raw(&self) -> u16 {
    self.y
  }

  /// Get the raw acceleration value in the Z-direction.
  #[inline(always)]
  pub fn z_raw(&self) -> u16 {
    self.z
  }

  /// Convert raw acceleration to g-force.
  fn raw_to_g(&self, acc: u16) -> f32 {
    (acc as i16) as f32 / self.mode.acceleration_sensitivity() as f32
  }

  /// Get the g-force in X-direction.
  #[inline]
  pub fn x_g(&self) -> f32 {
    self.raw_to_g(self.x)
  }

  /// Get the g-force in Y-direction.
  #[inline]
  pub fn y_g(&self) -> f32 {
    self.raw_to_g(self.y)
  }

  /// Get the g-force in Z-direction.
  #[inline]
  pub fn z_g(&self) -> f32 {
    self.raw_to_g(self.z)
  }

  /// Convert the acceleration to inclination angles.
  #[cfg(feature = "libm")]
  #[inline]
  pub fn to_inclination(&self) -> Inclination {
    let x_g = self.x_g();
    let y_g = self.y_g();
    let z_g = self.z_g();

    Inclination {
      x: Self::acc_to_inc(x_g, y_g, z_g),
      y: Self::acc_to_inc(y_g, x_g, z_g),
      z: Self::acc_to_inc(z_g, x_g, y_g),
    }
  }

  #[cfg(feature = "libm")]
  #[inline]
  fn acc_to_inc(a: f32, b: f32, c: f32) -> u16 {
    use core::f32::consts::FRAC_PI_2;
    use libm::{atan2f, powf, roundf, sqrtf};

    roundf(atan2f(a, sqrtf(powf(b, 2.0) + powf(c, 2.0))) * Inclination::FACTOR / FRAC_PI_2) as i16 as u16
  }
}

/// An inclination measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct Inclination {
  pub(crate) x: u16,
  pub(crate) y: u16,
  pub(crate) z: u16,
}

impl Inclination {
  pub(crate) const FACTOR: f32 = (1 << 14) as f32;

  /// Get the raw inclination value on the X-axis.
  #[inline(always)]
  pub fn x_raw(&self) -> u16 {
    self.x
  }

  /// Get the raw inclination value on the Y-axis.
  #[inline(always)]
  pub fn y_raw(&self) -> u16 {
    self.y
  }

  /// Get the raw inclination value on the Z-axis.
  #[inline(always)]
  pub fn z_raw(&self) -> u16 {
    self.z
  }

  #[inline]
  fn raw_to_degrees(raw: u16) -> f32 {
    raw as f32 / Inclination::FACTOR * 90.0
  }

  /// Get the inclination angle on the X-axis in degrees.
  #[inline]
  pub fn x_degrees(&self) -> f32 {
    Self::raw_to_degrees(self.x)
  }

  /// Get the inclination angle on the Y-axis in degrees.
  #[inline]
  pub fn y_degrees(&self) -> f32 {
    Self::raw_to_degrees(self.y)
  }

  /// Get the inclination angle on the Z-axis in degrees.
  #[inline]
  pub fn z_degrees(&self) -> f32 {
    Self::raw_to_degrees(self.z)
  }
}

/// A temperature measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
  pub(crate) temp: u16,
}

impl Temperature {
  /// Get the raw temperature value.
  #[inline(always)]
  pub fn raw(&self) -> u16 {
    self.temp
  }

  /// Get the temperature in °C.
  #[inline]
  pub fn degrees_celsius(&self) -> f32 {
    (self.temp as i16) as f32 / 18.9 - 273.0
  }
}

/// A self-test reading.
#[derive(Debug, Clone, PartialEq)]
pub struct SelfTest {
  pub(crate) sto: u16,
  pub(crate) mode: MeasurementMode,
}

impl SelfTest {
  /// Get the raw self-test value.
  pub fn raw(&self) -> u16 {
    self.sto
  }

  /// Check if the self-test reading is within the recommended thresholds.
  pub fn is_within_thresholds(&self) -> bool {
    self.mode.self_test_thresholds().contains(&(self.sto as i16))
  }
}

/// A component ID reading.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentId {
  pub(crate) id: u8,
}

impl ComponentId {
  /// The expected component ID.
  pub const WHOAMI: Self = Self { id: 0xC1 };

  /// Get the raw component ID.
  #[inline(always)]
  pub fn raw(&self) -> u8 {
    self.id
  }

  /// Check if the component ID is equal to the expected `WHOAMI` value.
  #[inline]
  pub fn is_correct(&self) -> bool {
    *self == Self::WHOAMI
  }
}

/// A serial number reading.
#[derive(Debug, Clone, PartialEq)]
pub struct Serial {
  pub(crate) part1: u16,
  pub(crate) part2: u16,
}

impl Serial {
  /// Get the serial number as an integer.
  pub const fn to_u32(&self) -> u32 {
    let [b0, b1] = self.part2.to_be_bytes();
    let [b2, b3] = self.part1.to_be_bytes();
    u32::from_be_bytes([b0, b1, b2, b3])
  }
}

impl fmt::Display for Serial {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:010}B33", self.to_u32())
  }
}

bitflags! {
  /// `STATUS` register flags.
  pub struct Status: u16 {
    /// Digital block error type 1
    const DIGI1          = 0b1000000000;
    /// Digital block error type 2
    const DIGI2          = 0b0100000000;
    /// Clock error
    const CLK            = 0b0010000000;
    /// Signal saturated in signal path
    const SAT            = 0b0001000000;
    /// Temperature signal path saturated
    const TEM_SAT        = 0b0000100000;
    /// Start-up indication or voltage level failure
    const PWR            = 0b0000010000;
    /// Error in non-volatile memory
    const MEM            = 0b0000001000;
    /// Device in power down mode
    const PD             = 0b0000000100;
    /// Operation mode changed
    const MODE_CHANGE    = 0b0000000010;
    /// Component internal connection error
    const PIN_CONTINUITY = 0b0000000001;
  }
}

bitflags! {
  /// `ERR_FLAG1` register flags.
  pub struct Error1: u16 {
    /// Signal saturated at A2D
    const ADC_SAT    = 0b100000000000;
    /// Signal saturated at C2V
    const AFE_SAT    = 0b011111111110;
    /// Error in non-volatile memory
    const MEM        = 0b000000000001;
  }
}

bitflags! {
  /// `ERR_FLAG2` register flags.
  pub struct Error2: u16 {
    /// External capacitor connection error
    const D_EXT_C      = 0b10000000000000;
    /// External capacitor connection error
    const A_EXT_C      = 0b01000000000000;
    /// Analog ground connection error
    const AGND         = 0b00100000000000;
    /// Supply voltage error
    const VDD          = 0b00010000000000;
    /// Operation mode changed by user
    const MODE_CHANGE  = 0b00001000000000;
    /// Device in power down mode
    const PD           = 0b00000100000000;
    /// Memory CRC check failed
    const MEMORY_CRC   = 0b00000010000000;
    /// Analog power error
    const APWR         = 0b00000000100000;
    /// After start-up or reset:
    /// This flag is set high. No actions needed.
    ///
    /// During normal operation:
    /// Digital power error. Component failure possible.
    /// SW or HW reset needed.
    const DPWR         = 0b00000000010000;
    /// Reference voltage error
    const VREF         = 0b00000000001000;
    /// Analog power error
    const APWR_2       = 0b00000000000100;
    /// Temperature signal path saturated
    const TEMP_SAT     = 0b00000000000010;
    /// Clock error
    const CLK          = 0b00000000000001;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_acceleration() {
    let acceleration = Acceleration { x: 0x00DC, y: 0, z: 0, mode: MeasurementMode::FullScale12 };
    let precision = 10000.0;
    assert_eq!((acceleration.x_g() * precision).round() / precision, 0.0367);
  }

  #[test]
  fn test_inclination() {
    let inclination = Inclination { x: 0x0F88, y: 0, z: 0 };
    let precision = 100.0;
    assert_eq!((inclination.x_degrees() * precision).round() / precision, 21.84);
  }

  #[test]
  fn test_temperature() {
    let temperature = Temperature { temp: 0x161E };
    let precision = 10.0;
    assert_eq!((temperature.degrees_celsius() * precision).round() / precision, 26.6);
  }

  #[test]
  fn test_serial() {
    let serial = Serial { part1: 0xF7DA, part2: 0x3CE5 };
    assert_eq!(serial.to_string(), "1021704154B33");
  }

  #[test]
  fn test_serial_empty() {
    let serial = Serial { part1: 0, part2: 0 };
    assert_eq!(serial.to_string(), "0000000000B33");
  }
}
