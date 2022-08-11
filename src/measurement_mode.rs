use core::{num::NonZeroU32, ops::RangeInclusive};

/// A measurement mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeasurementMode {
  /// 1.2g full-scale,
  /// 40 Hz first-order low-pass filter
  FullScale12,
  /// 2.4g full-scale,
  /// 70 Hz first-order low-pass filter
  FullScale24,
  /// Inclination mode,
  /// 10 Hz first-order low-pass filter
  Inclination,
  /// Inclination (low noise) mode,
  /// 10 Hz first-order low-pass filter,
  InclinationLowNoise,
}

impl Default for MeasurementMode {
  fn default() -> Self {
    Self::new()
  }
}

impl MeasurementMode {
  pub(crate) const fn new() -> Self {
    Self::FullScale12
  }

  pub(crate) const fn self_test_thresholds(&self) -> RangeInclusive<i16> {
    match self {
      Self::FullScale12 => -1800..=1800,
      Self::FullScale24 => -900..=900,
      Self::Inclination | Self::InclinationLowNoise => -3600..=3600,
    }
  }

  pub(crate) const fn acceleration_sensitivity(&self) -> u16 {
    match self {
      Self::FullScale12 => 6000,
      Self::FullScale24 => 3000,
      Self::Inclination | Self::InclinationLowNoise => 12000,
    }
  }

  pub(crate) const fn start_up_wait_time_us(&self) -> NonZeroU32 {
    match self {
      // SAFTEY: 25000 is not 0.
      MeasurementMode::FullScale12 => unsafe {
        NonZeroU32::new_unchecked(25_000)
      },
      // SAFTEY: 15000 is not 0.
      MeasurementMode::FullScale24 => unsafe {
        NonZeroU32::new_unchecked(15_000)
      },
      // SAFTEY: 100000 is not 0.
      MeasurementMode::Inclination | MeasurementMode::InclinationLowNoise => unsafe {
        NonZeroU32::new_unchecked(100_000)
      },
    }
  }
}
