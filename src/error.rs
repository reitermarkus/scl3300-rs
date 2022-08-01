#[derive(Debug)]
pub enum Error<E> {
  /// Startup error
  Startup,
  /// ReturnStatus error
  ReturnStatus,
  /// CRC checksum mismatch
  Crc,
  Spi(E),
}
