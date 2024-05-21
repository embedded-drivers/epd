//! IL3895 driver.

use crate::interface::{DisplayError, DisplayInterface};
use embedded_graphics::pixelcolor::Gray4;
use embedded_hal::delay::DelayNs;

use super::{Driver, FastUpdateDriver, GrayScaleDriver, MultiColorDriver, WaveformDriver};

/// 150 source outputs, 250 gate outputs, B/W
/// 30 bytes LUT, format is different from SSD1608.
/// Command payload bytes is different from SSD1608.
// https://gitee.com/andelf/epd-playground/blob/master/src/utility/EPD_2in13.cpp
/// 2in13 B/W 122x250
pub struct IL3895;

impl Driver for IL3895 {
    type Error = DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 200_000, 200_000);
        Self::busy_wait(di)?;

        di.send_command_data(0x2C, &[0xA8])?;

        di.send_command_data(0x3a, &[0x1a])?; // set dummy line period
        di.send_command_data(0x3b, &[0x08])?; // set gate line width
        di.send_command_data(0x3c, &[0x63])?; // border waveform control

        di.send_command_data(0x11, &[0b011])?; // data entry mode: default

        // LUT is required
        #[rustfmt::skip]
        const LUT_FULL_UPDATE: [u8; 30] = [
            // VS
            0x22, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x11, 0x00, 0x00,
            // PADDING
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // RP TP
            0x1E, 0x1E,
            0x1E, 0x1E,
            0x1E, 0x1E,
            0x1E, 0x1E,
            0x01, 0x00,
            // PADDING
            0x00, 0x00, 0x00,
            // R3A_A, dummy line
            0x00,
        ];
        di.send_command_data(0x32, &LUT_FULL_UPDATE)?;

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        // Driver Output control
        di.send_command_data(0x01, &[((y - 1) & 0xff) as u8, 0])?;

        // set ram x start/end
        di.send_command_data(0x44, &[0, ((x - 1) >> 3) as u8])?;
        // set ram y start/end
        di.send_command_data(0x45, &[0, ((y - 1) & 0xff) as u8])?;

        Ok(())
    }

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        // set cursor
        di.send_command_data(0x4E, &[0])?;
        di.send_command_data(0x4f, &[0])?;

        // write ram
        di.send_command(0x24)?;
        di.send_data_from_iter(buffer)?;

        di.send_command(0xff)?;
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xc4])?;
        di.send_command(0x20)?;
        di.send_command(0xff)?;

        Self::busy_wait(di)?;
        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        _delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x10, &[0x01])?;
        Ok(())
    }
}

impl WaveformDriver for IL3895 {
    fn update_waveform<DI: DisplayInterface>(
        di: &mut DI,
        lut: &'static [u8],
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x32, lut)?;
        Ok(())
    }
}

impl FastUpdateDriver for IL3895 {
    fn setup_fast_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // LUT is required
        #[rustfmt::skip]
        const LUT_FULL_UPDATE: [u8; 30] = [
            // VS
            0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // PADDING
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // RP TP
            0x0F, 0x01,
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x00,
            // PADDING
            0x00, 0x00, 0x00,
            // R3A_A, dummy line
            0x00,
        ];
        di.send_command_data(0x32, &LUT_FULL_UPDATE)?;

        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // LUT is required
        #[rustfmt::skip]
        const LUT_FULL_UPDATE: [u8; 30] = [
            // VS
            0x22, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x11, 0x00, 0x00,
            // PADDING
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // RP TP
            0x1E, 0x1E,
            0x1E, 0x1E,
            0x1E, 0x1E,
            0x1E, 0x1E,
            0x01, 0x00,
            // PADDING
            0x00, 0x00, 0x00,
            // R3A_A, dummy line
            0x00,
        ];
        di.send_command_data(0x32, &LUT_FULL_UPDATE)?;

        Ok(())
    }
}
