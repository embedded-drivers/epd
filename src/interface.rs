//! The display interface for e-Paper displays.

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

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

    fn is_busy_on(&self) -> bool;

    /// Hard reset
    fn reset<D>(&mut self, delay: &mut D, initial_delay: u32, duration: u32)
    where
        D: DelayUs<u32>;
}

/// EPaperDisplay SPI display interface.
pub struct EPDInterface<SPI, CS, DC, RST, BUSY> {
    spi: SPI,
    cs: CS,
    dc: DC,
    rst: RST,
    busy: BUSY,
}

impl<SPI, CS, DC, RST, BUSY> EPDInterface<SPI, CS, DC, RST, BUSY>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    pub fn new(spi: SPI, cs: CS, dc: DC, rst: RST, busy: BUSY) -> Self {
        EPDInterface {
            spi,
            cs,
            dc,
            rst,
            busy,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC, CS, BUSY) {
        (self.spi, self.dc, self.cs, self.busy)
    }
}

impl<SPI, CS, DC, RST, BUSY> DisplayInterface for EPDInterface<SPI, CS, DC, RST, BUSY>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
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

    fn send_data_from_iter<'a, I>(&mut self, iter: I) -> Result<usize, DisplayError>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        self.cs.set_low().map_err(|_| DisplayError::CSError)?;
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;

        let mut n = 0;
        for &d in iter {
            n += 1;
            let ret = self
                .spi
                .write(&[d])
                .map_err(|_| DisplayError::BusWriteError);
            if ret.is_err() {
                self.cs.set_high().ok();
                ret?; // return the error
            }
        }

        // Deassert chip select pin
        self.cs.set_high().ok();

        Ok(n)
    }

    fn reset<D>(&mut self, delay: &mut D, initial_delay: u32, duration: u32)
    where
        D: DelayUs<u32>,
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

    fn is_busy_on(&self) -> bool {
        self.busy.is_high().unwrap_or(false)
    }
}

/// EPaperDisplay SPI display interface.
pub struct EPDInterfaceNoCS<SPI, DC, RST, BUSY> {
    spi: SPI,
    dc: DC,
    rst: RST,
    busy: BUSY,
}

impl<SPI, DC, RST, BUSY> EPDInterfaceNoCS<SPI, DC, RST, BUSY>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    pub fn new(spi: SPI, dc: DC, rst: RST, busy: BUSY) -> Self {
        EPDInterfaceNoCS { spi, dc, rst, busy }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (SPI, DC, BUSY) {
        (self.spi, self.dc, self.busy)
    }
}

impl<SPI, DC, RST, BUSY> DisplayInterface for EPDInterfaceNoCS<SPI, DC, RST, BUSY>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
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

    fn is_busy_on(&self) -> bool {
        self.busy.is_high().unwrap_or(false)
    }

    fn reset<D>(&mut self, delay: &mut D, initial_delay: u32, duration: u32)
    where
        D: DelayUs<u32>,
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
