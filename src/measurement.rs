use core::fmt;

use crate::MeasurementMode;

/// Acceleration measurement
#[derive(Debug, Clone, PartialEq)]
pub struct Acceleration {
  pub(crate) x: u16,
  pub(crate) y: u16,
  pub(crate) z: u16,
  pub(crate) mode: MeasurementMode,
}

impl Acceleration {
  /// Create a new `Acceleration` for storing a measurement.
  pub const fn new() -> Self {
    Self { x: 0, y: 0, z: 0, mode: MeasurementMode::new() }
  }

  /// Raw acceleration value in X direction.
  #[inline(always)]
  pub fn x_raw(&self) -> u16 {
    self.x
  }

  /// Raw acceleration value in Y direction.
  #[inline(always)]
  pub fn y_raw(&self) -> u16 {
    self.y
  }

  /// Raw acceleration value in Z direction.
  #[inline(always)]
  pub fn z_raw(&self) -> u16 {
    self.z
  }

  /// Convert raw acceleration to g-force.
  fn convert_acc(&self, acc: u16) -> f32 {
    (acc as i16) as f32 / self.mode.acceleration_sensitivity() as f32
  }

  /// G-force in X direction.
  #[inline]
  pub fn x_g(&self) -> f32 {
    self.convert_acc(self.x)
  }

  /// G-force in Y direction.
  #[inline]
  pub fn y_g(&self) -> f32 {
    self.convert_acc(self.y)
  }

  /// G-force in Z direction.
  #[inline]
  pub fn z_g(&self) -> f32 {
    self.convert_acc(self.z)
  }

  /// Calculate inclination from acceleration values.
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

  #[inline]
  fn acc_to_inc(a: f32, b: f32, c: f32) -> u16 {
    use core::f32::consts::FRAC_PI_2;
    use libm::{atan2f, powf, roundf, sqrtf};

    roundf(atan2f(a, sqrtf(powf(b, 2.0) + powf(c, 2.0))) * Inclination::FACTOR / FRAC_PI_2) as i16 as u16
  }
}

/// Inclination measurement
#[derive(Debug, Clone, PartialEq)]
pub struct Inclination {
  pub(crate) x: u16,
  pub(crate) y: u16,
  pub(crate) z: u16,
}

impl Inclination {
  pub(crate) const FACTOR: f32 = (1 << 14) as f32;

  /// Create a new `Inclination` for storing a measurement.
  pub const fn new() -> Self {
    Self { x: 0, y: 0, z: 0 }
  }

  /// Raw inclination angle on X-axis.
  #[inline(always)]
  pub fn x_raw(&self) -> u16 {
    self.x
  }

  /// Inclination angle on X-axis in degrees
  #[inline]
  pub fn x_degrees(&self) -> f32 {
    Self::raw_to_degrees(self.x)
  }

  /// Raw inclination angle on Y-axis.
  #[inline(always)]
  pub fn y_raw(&self) -> u16 {
    self.y
  }

  /// Inclination angle on Y-axis in degrees
  #[inline]
  pub fn y_degrees(&self) -> f32 {
    Self::raw_to_degrees(self.y)
  }

  /// Raw inclination angle on Z-axis.
  #[inline(always)]
  pub fn z_raw(&self) -> u16 {
    self.z
  }

  /// Inclination angle on Z-axis in degrees
  #[inline]
  pub fn z_degrees(&self) -> f32 {
    Self::raw_to_degrees(self.z)
  }

  #[inline]
  fn raw_to_degrees(raw: u16) -> f32 {
    raw as f32 / Inclination::FACTOR * 90.0
  }
}

/// Temperature measurement
#[derive(Debug, Clone, PartialEq)]
pub struct Temperature {
  pub(crate) temp: u16,
}

impl Temperature {
  /// Create a new `Temperature` for storing a measurement.
  pub const fn new() -> Self {
    Self { temp: 0 }
  }

  /// Raw temperature.
  #[inline(always)]
  pub fn raw(&self) -> u16 {
    self.temp
  }

  /// Temperature in °C.
  #[inline]
  pub fn degrees_celsius(&self) -> f32 {
    (self.temp as i16) as f32 / 18.9 - 273.0
  }
}

/// Self-test measurement
#[derive(Debug, Clone, PartialEq)]
pub struct SelfTest {
  pub(crate) sto: u16,
  pub(crate) mode: MeasurementMode,
}

impl SelfTest {
  /// Create a new `SelfTest` for storing a measurement.
  pub const fn new() -> Self {
    Self { sto: 0, mode: MeasurementMode::new() }
  }

  /// Raw self-test value.
  pub fn raw(&self) -> u16 {
    self.sto
  }

  pub fn is_within_thresholds(&self) -> bool {
    self.mode.self_test_thresholds().contains(&(self.sto as i16))
  }
}

/// Temperature measurement
#[derive(Debug, Clone, PartialEq)]
pub struct WhoAmI {
  pub(crate) whoami: u16,
}

impl WhoAmI {
  /// Create a new `Temperature` for storing a measurement.
  pub const fn new() -> Self {
    Self { whoami: 0 }
  }

  /// Raw `WHOAMI` value.
  #[inline(always)]
  pub fn raw(&self) -> u16 {
    self.whoami
  }

  /// Check if the value matches the expected value.
  #[inline]
  pub fn check(&self) -> bool {
    self.whoami == 0xC1
  }
}

/// Temperature measurement
#[derive(Debug, Clone, PartialEq)]
pub struct Serial {
  pub(crate) part1: u16,
  pub(crate) part2: u16,
}

impl Serial {
  /// Create a new `Temperature` for storing a measurement.
  pub const fn new() -> Self {
    Self { part1: 0, part2: 0 }
  }
}

impl fmt::Display for Serial {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let [b0, b1] = self.part2.to_be_bytes();
    let [b2, b3] = self.part1.to_be_bytes();
    let serial = u32::from_be_bytes([b0, b1, b2, b3]);

    write!(f, "{}B33", serial)
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
}
