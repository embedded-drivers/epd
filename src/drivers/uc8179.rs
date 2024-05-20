//! UC8176 driver
//!
//! Up to 20MHz

use embedded_hal::delay::DelayNs;

use super::{Driver, MultiColorDriver};
use crate::interface::{DisplayError, DisplayInterface};

/// 800 x 600 x 2
pub struct UC8179;

impl Driver for UC8179 {
    type Error = DisplayError;
    // const BLACK_BIT: bool = true;

    fn busy_wait<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command(0x71)?; // read status

        while !di.is_busy_on() {}
        Ok(())
    }

    fn wake_up<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 10_000, 10_000); // HW Reset
        Self::busy_wait(di)?;

        // Power Setting
        // VGH=20V, VGL=-20V, VDH=15V, VDL=-15V
        di.send_command_data(0x01, &[0x07, 0x07, 0x3f, 0x3f])?;

        //        di.send_command_data(0x06, &[0x17, 0x17, 0x17])?;

        di.send_command(0x04)?; // power on
        Self::busy_wait(di)?;

        // Panel setting
        // KW-3f   KWR-2F BWROTP 0f BWOTP 1f
        di.send_command_data(0x00, &[0x0F])?;

        di.send_command_data(0x15, &[0x00])?;

        di.send_command_data(0x50, &[0x11, 0x07])?; // VCOM AND DATA INTERVAL SETTING

        di.send_command_data(0x60, &[0x22])?; // TCON SETTING

        // di.send_command_data(0x00, &[0x3f])?; // panel setting

        //        di.send_command_data(0x30, &[0x3c])?; // PLL control

        //      di.send_command_data(0x82, &[0x12])?; // VCM_DC setting
        //        di.send_command_data(0x50, &[0x97])?; // VCOM AND DATA INTERVAL SETTING

        // fill r channel with zeros(white)
        // di.send_command(0x13)?;
        //        di.send_data_from_iter(iter::repeat(&0x00).take(400 * 300 / 8))?;

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        di.send_command_data(0x61, &[(x >> 8) as u8, x as u8, (y >> 8) as u8, y as u8])?;
        Ok(())
    }

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        di.send_command(0x10)?;
        di.send_data_from_iter(buffer)?;
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command(0x04)?; // Power on
        Self::busy_wait(di)?;

        //   di.send_command(0x12)?; // display refresh

        Self::busy_wait(di)?;

        Ok(())
    }
}

impl MultiColorDriver for UC8179 {
    fn update_channel_frame<'a, DI: DisplayInterface, I>(
        di: &mut DI,
        channel: u8,
        buffer: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        if channel == 0 {
            di.send_command(0x10)?;
            di.send_data_from_iter(buffer)?;
        } else if channel == 1 {
            di.send_command(0x13)?;
            di.send_data_from_iter(buffer)?;
        } else {
            return Err(DisplayError::InvalidChannel);
        }

        Ok(())
    }
}
