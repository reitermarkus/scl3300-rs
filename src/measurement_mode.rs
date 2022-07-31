#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeasurementMode {
  /// 1.2g full-scale, 40 Hz 1st order low pass filter
  FullScale12,
  /// 2.4g full-scale, 70 Hz 1st order low pass filter
  FullScale24,
  /// Inclination mode, 10 Hz 1st order low pass filter
  Inclination,
  /// Inclination mode, 10 Hz 1st order low pass filter, Low noise mode
  InclinationLowNoise,
}

impl Default for MeasurementMode {
  fn default() -> Self {
    Self::FullScale12
  }
}

impl MeasurementMode {
  pub const fn sto_thresholds(&self) -> (i16, i16) {
    match self {
      Self::FullScale12 => (-1800, 1800),
      Self::FullScale24 => (-900, 900),
      Self::Inclination | Self::InclinationLowNoise => (-3600, 3600),
    }
  }

  pub const fn acceleration_sensitivity(&self) -> (u16, u8) {
    match self {
      Self::FullScale12 => (6000, 105),
      Self::FullScale24 => (3000, 52),
      Self::Inclination | Self::InclinationLowNoise => (12000, 52),
    }
  }

  pub const fn inclination_sensitivity(&self) -> u8 {
    182
  }

  pub const fn wait_time(&self) -> u8 {
    match self {
      Self::FullScale12 => 25,
      Self::FullScale24 => 15,
      Self::Inclination | Self::InclinationLowNoise => 100,
    }
  }

  pub fn convert_acceleration(&self, acc: u16) -> f32 {
    (acc as i16) as f32 / self.acceleration_sensitivity().0 as f32
  }

  pub fn convert_angle(&self, ang: u16) -> f32 {
    (ang as i16) as f32 / (1u16 << 14) as f32 * 90.0
  }

  pub fn convert_temperature(&self, temp: u16) -> f32 {
    (temp as i16) as f32 / 18.9 - 273.0
  }
}
