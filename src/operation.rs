use crate::{Frame, MeasurementMode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bank {
  /// Bank 0
  Zero,
  /// Bank 1
  One,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Output {
  AccelerationX,
  AccelerationY,
  AccelerationZ,
  AngleX,
  AngleY,
  AngleZ,
  Temperature,
  SelfTest,
  Status,
  Error1,
  Error2,
  // No need to use this for now since the library keeps track of this implicitly.
  #[allow(unused)]
  Command,
  WhoAmI,
  Serial1,
  Serial2,
  // No need to use this for now since switching banks is only done in one place.
  #[allow(unused)]
  CurrentBank,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
  Read(Output),
  EnableAngleOutputs,
  ChangeMode(MeasurementMode),
  PowerDown,
  WakeUp,
  Reset,
  SwitchBank(Bank),
}

impl Operation {
  pub(crate) const fn to_frame(self) -> Frame {
    use Operation::*;
    use Output::*;
    use MeasurementMode::*;
    use Bank::*;

    let frame: u32 = match self {
      Read(AccelerationX)             => 0x040000F7,
      Read(AccelerationY)             => 0x080000FD,
      Read(AccelerationZ)             => 0x0C0000FB,
      Read(SelfTest)                  => 0x100000E9,
      EnableAngleOutputs              => 0xB0001F6F,
      Read(AngleX)                    => 0x240000C7,
      Read(AngleY)                    => 0x280000CD,
      Read(AngleZ)                    => 0x2C0000CB,
      Read(Temperature)               => 0x140000EF,
      Read(Status)                    => 0x180000E5,
      Read(Error1)                    => 0x1C0000E3,
      Read(Error2)                    => 0x200000C1,
      Read(Command)                   => 0x340000DF,
      ChangeMode(FullScale12)         => 0xB400001F,
      ChangeMode(FullScale24)         => 0xB4000102,
      ChangeMode(Inclination)         => 0xB4000225,
      ChangeMode(InclinationLowNoise) => 0xB4000338,
      PowerDown                       => 0xB400046B,
      WakeUp                          => 0xB400001F,
      Reset                           => 0xB4002098,
      Read(WhoAmI)                    => 0x40000091,
      Read(Serial1)                   => 0x640000A7,
      Read(Serial2)                   => 0x680000AD,
      Read(CurrentBank)               => 0x7C0000B3,
      SwitchBank(Zero)                => 0xFC000073,
      SwitchBank(One)                 => 0xFC00016E,
    };

    Frame { bytes: frame.to_be_bytes() }
  }
}
