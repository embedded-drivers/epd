//! SSD1680 driver
//!
//! For:
//! - GDEY029Z94

// 153 bytes LUT.

use core::iter;

use super::{Driver, MultiColorDriver};
use crate::interface::{DisplayError, DisplayInterface};

use embedded_hal::blocking::delay::DelayUs;
/// 176 Source x 296 Gate Red/Black/White
pub struct SSD1680;

impl Driver for SSD1680 {
    type Error = DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 10_000, 10_000); // HW Reset
        Self::busy_wait(di)?;

        di.send_command(0x12)?; // swreset
        di.busy_wait();

        di.send_command_data(0x01, &[0x27, 0x01, 0x00])?; // Driver output control

        di.send_command_data(0x11, &[0b0_11])?; // data entry mode

        di.send_command_data(0x21, &[0x00, 0x80])?; // Display update control

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        // Set RAM X - address Start / End position
        di.send_command_data(0x44, &[0x00, ((x - 1) >> 3) as u8])?;
        // Set RAM Y - address Start / End position
        di.send_command_data(
            0x45,
            &[0x00, 0x00, ((y - 1) & 0xff) as u8, ((y - 1) >> 8) as u8],
        )?;
        Ok(())
    }

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

        di.send_command(0x24)?;
        let n = di.send_data_from_iter(buffer)?;
        di.send_command(0x7f)?; // NOP

        // fill R frame with zeros(white)
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

        di.send_command(0x26)?;
        di.send_data_from_iter(iter::repeat(&0).take(n))?;

        di.send_command(0x7f)?; // NOP

        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xf7])?;
        di.send_command(0x20)?;
        Self::busy_wait(di)?;

        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x10, &[0x01])?;
        delay.delay_us(100_000);
        Ok(())
    }
}

impl MultiColorDriver for SSD1680 {
    fn update_channel_frame<'a, DI: DisplayInterface, I>(
        di: &mut DI,
        channel: u8,
        buffer: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

        if channel == 0 {
            di.send_command(0x24)?;
            di.send_data_from_iter(buffer)?;
        } else if channel == 1 {
            di.send_command(0x26)?;
            di.send_data_from_iter(buffer)?;
        } else {
        }
        Ok(())
    }
}
