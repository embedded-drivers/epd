//! The display interface for e-Paper displays.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

#[derive(Clone, Debug)]
pub enum DisplayError {
    InvalidFormatError,
    BusWriteError,
    DCError,
    CSError,
    BUSYError,
    InvalidChannel,
}

/// Trait implemented by displays to provide implemenation of core functionality.
pub trait DisplayInterface {
    fn send_command_data(&mut self, command: u8, data: &[u8]) -> Result<(), DisplayError> {
        self.send_command(command)?;
        self.send_data(data)?;
        Ok(())
    }

    /// Send a command to the controller.
    fn send_command(&mut self, command: u8) -> Result<(), DisplayError>;

    /// Send data for a command.
    fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError>;

    /// Send data via iter
    fn send_data_from_iter<'a, I>(&mut self, iter: I) -> Result<usize, DisplayError>
    where
        I: IntoIterator<Item = &'a u8>;

    fn is_busy_on(&mut self) -> bool;

    /// Hard reset
    fn reset<D>(&mut self, delay: &mut D, initial_delay: u32, duration: u32)
    where
        D: DelayNs;
}

/// E-Paper Display SPI display interface.
pub struct EpdInterface<SPI, DC, RST, BUSY> {
    spi: SPI,
    dc: DC,
    rst: RST,
    busy: BUSY,
}

impl<SPI, DC, RST, BUSY> EpdInterface<SPI, DC, RST, BUSY>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    pub fn new(spi: SPI, dc: DC, rst: RST, busy: BUSY) -> Self {
        EpdInterface { spi, dc, rst, busy }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC, BUSY) {
        (self.spi, self.dc, self.busy)
    }
}

impl<SPI, DC, RST, BUSY> DisplayInterface for EpdInterface<SPI, DC, RST, BUSY>
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    /// Send a command to the controller.
    fn send_command(&mut self, command: u8) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let ret = self
            .spi
            .write(&[command])
            .map_err(|_| DisplayError::BusWriteError);

        ret
    }

    /// Send data for a command.
    fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        // 1 = data, 0 = command
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        // Send words over SPI
        let ret = self
            .spi
            .write(data)
            .map_err(|_| DisplayError::BusWriteError);

        ret
    }

    fn send_data_from_iter<'a, I>(&mut self, iter: I) -> Result<usize, DisplayError>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        let mut n = 0;
        for &d in iter {
            n += 1;
            self.spi
                .write(&[d])
                .map_err(|_| DisplayError::BusWriteError)?;
        }

        Ok(n)
    }

    fn is_busy_on(&mut self) -> bool {
        self.busy.is_high().unwrap_or(false)
    }

    fn reset<D>(&mut self, delay: &mut D, initial_delay: u32, duration: u32)
    where
        D: DelayNs,
    {
        let _ = self.rst.set_high();
        delay.delay_us(initial_delay);

        let _ = self.rst.set_low();
        delay.delay_us(duration);
        let _ = self.rst.set_high();
        //TODO: the upstream libraries always sleep for 200ms here
        // 10ms works fine with just for the 7in5_v2 but this needs to be validated for other devices
        delay.delay_us(200_000);
    }
}
