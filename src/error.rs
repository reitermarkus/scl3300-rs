/// An SCL3300 error.
#[derive(Debug)]
pub enum Error<E> {
  /// Startup error
  Startup,
  /// ReturnStatus error
  ReturnStatus,
  /// CRC checksum mismatch
  Crc,
  /// SPI error
  Spi(E),
}
