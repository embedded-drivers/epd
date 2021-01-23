//! The display interface for e-Paper displays.

use embedded_hal::digital::v2::{InputPin, OutputPin};

#[derive(Clone, Debug)]
pub enum DisplayError {
    InvalidFormatError,
    BusWriteError,
    DCError,
    CSError,
    BUSYError,
}

/// Trait implemented by displays to provide implemenation of core functionality.
pub trait DisplayInterface {
    /// Send a command to the controller.
    fn send_command(&mut self, command: u8) -> Result<(), DisplayError>;

    /// Send data for a command.
    fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError>;

    /// Wait for the controller to indicate it is not busy.
    fn busy_wait(&self);
}

/// EPaperDisplay SPI display interface.
pub struct Interface<SPI, CS, DC, BUSY> {
    spi: SPI,
    cs: CS,
    dc: DC,
    busy: BUSY,
}

impl<SPI, CS, DC, BUSY> Interface<SPI, CS, DC, BUSY>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    BUSY: InputPin,
{
    pub fn new(spi: SPI, dc: DC, cs: CS, busy: BUSY) -> Self {
        Interface { spi, dc, cs, busy }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC, CS, BUSY) {
        (self.spi, self.dc, self.cs, self.busy)
    }
}

impl<SPI, CS, DC, BUSY> DisplayInterface for Interface<SPI, CS, DC, BUSY>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    BUSY: InputPin,
{
    /// Send a command to the controller.
    fn send_command(&mut self, command: u8) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let ret = self
            .spi
            .write(&[command])
            .map_err(|_| DisplayError::BusWriteError);

        // Deassert chip select pin
        self.cs.set_high().ok();

        ret
    }

    /// Send data for a command.
    fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        // Assert chip select pin
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;

        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let ret = self
            .spi
            .write(data)
            .map_err(|_| DisplayError::BusWriteError);

        // Deassert chip select pin
        self.cs.set_high().ok();

        ret
    }

    /// Wait for the controller to indicate it is not busy.
    fn busy_wait(&self) {
        // LOW: idle, HIGH: busy
        while self.busy.is_high().unwrap_or(false) {}
    }
}
