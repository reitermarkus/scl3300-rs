use bitflags::bitflags;

bitflags! {
  pub struct Status: u16 {
    const PIN_CONTINUITY = 0b0000000001;
    const MODE_CHANGE    = 0b0000000010;
    const PD             = 0b0000000100;
    const MEM            = 0b0000001000;
    const PWR            = 0b0000010000;
    const TEM_SAT        = 0b0000100000;
    const SAT            = 0b0001000000;
    const CLK            = 0b0010000000;
    const DIGI2          = 0b0100000000;
    const DIGI1          = 0b1000000000;
  }
}

bitflags! {
  pub struct Error1: u16 {
    const MEM        = 0b000000000001;
    const AFE_SAT    = 0b011111111110;
    const ADC_SAT    = 0b100000000000;
  }
}

bitflags! {
  pub struct Error2: u16 {
    const CLK          = 0b00000000000001;
    const TEMP_SAT     = 0b00000000000010;
    const APWR_2       = 0b00000000000100;
    const VREF         = 0b00000000001000;
    const DPWR         = 0b00000000010000;
    const APWR         = 0b00000000100000;
    const MEMORY_CRC   = 0b00000010000000;
    const PD           = 0b00000100000000;
    const MODE_CHANGE  = 0b00001000000000;
    const VDD          = 0b00010000000000;
    const AGND         = 0b00100000000000;
    const A_EXT_C      = 0b01000000000000;
    const D_EXT_C      = 0b10000000000000;
  }
}
