use bitflags::bitflags;

bitflags! {
  /// `STATUS` register flags
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
  /// `ERR_FLAG1` register flags
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
  /// `ERR_FLAG2` register flags
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
    /// After star-up or reset:
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
