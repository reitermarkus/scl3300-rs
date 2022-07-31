#[derive(Debug)]
pub enum Error<E> {
  /// Startup error
  Startup,
  /// CRC checksum mismatch
  Crc,
  Spi(E),
}
