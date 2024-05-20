use core::iter;

use crate::interface::{DisplayError, DisplayInterface};
use embedded_graphics::pixelcolor::Gray4;
use embedded_hal::delay::DelayNs;

use super::{Driver, FastUpdateDriver, GrayScaleDriver, MultiColorDriver, WaveformDriver};

/// By guessing, it's like the IL0373, but with different resulution.
/// Up to 160 source x 296 gate resolution
/// small, including 420 and 437
/// Pervasive Displays, small up to 4.37
// https://github.com/rei-vilo/PDLS_EXT3_Basic/blob/main/src/Screen_EPD_EXT3.cpp
pub struct PervasiveDisplays;

impl Driver for PervasiveDisplays {
    type Error = DisplayError;

    fn busy_wait<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // negative logic
        while !di.is_busy_on() {}
        Ok(())
    }

    fn wake_up<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 10_000, 10_000);
        Self::busy_wait(di)?;

        // panel setting
        // 0b0000_1110 0x0e
        // 0bxxxx_xxx
        // 0b0010_0000: use LUT from register
        // 0b0001_0000: B/W mode, use LU1 only
        // 0b0000_1000: scan up
        // 0b0000_0100: scan right
        // 0b0000_0000: scan right
        // ob0000_0001: Disable power
        di.send_command_data(0x00, &[0xbf])?; // soft-reset

        delay.delay_us(5_000_u32);
        di.send_command_data(0xe5, &[0x19]).unwrap(); // Input Temperature 0°C = 0x00, 22°C = 0x16, 25°C = 0x19

        di.send_command_data(0xe0, &[0x02]).unwrap(); // Active Temperature

        #[rustfmt::skip]
        const LUT_VCOM: [u8; 44] = [
            // Level select, Number of Frames 0-3, Times to repeat
            0x00, 0x00, 0x00, 0x0A, 0x00, 0x00,
            0x00, 0x01, 0x60, 0x14, 0x14, 0x00,
            0x00, 0x01, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x13, 0x0A, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ST_XON[6:0], ST_CHV[6:0]
            0x00, 0x00,
        ];
        // white to white
        #[rustfmt::skip]
        const LUT_WW: [u8; 42] = [
            0b10_00_01_10, 0x06, 0x00, 0x03, 0x25, 0x01,
            0x90, 0x00, 0x14, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x0A, 0x00, 0x00, 0x00,
            0xA0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        // black to white
        #[rustfmt::skip] // LUT_R
        const LUT_BW: [u8; 42] = [
            0b01_10_01_10, 0x0f, 0x0f, 0x0f, 0x1f, 0x00,
            0b10_00_01_10, 0x06, 0x00, 0x03, 0x25, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        #[rustfmt::skip] // x LUT_W
        const LUT_WB: [u8; 42] = [
            // LEVEL, frame0, frame1, frame2, frame3, RP
            0b10_00_01_10, 0x06, 0x00, 0x03, 0x25, 0x01,
            0b01_10_01_10, 0x0f, 0x0f, 0x0f, 0x1f, 0x00,
            0x00, 0x14, 0x0A, 0x00, 0x00, 0x00,
            0x99, 0x0B, 0x04, 0x04, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        #[rustfmt::skip] // x LUT_B
        const LUT_BB: [u8; 42] = [
            0b10_01_10_01, 0x06, 0x06, 0x02, 0x2c, 0x01,
            0b01_10_01_10, 0x0f, 0x0f, 0x0f, 0x1f, 0x00,
            0x20, 0x14, 0x0A, 0x00, 0x00, 0x00,
            0x50, 0x13, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // LUTC
        di.send_command_data(0x20, &LUT_VCOM)?;
        // LUTWW
        di.send_command_data(0x21, &LUT_WW)?;
        // LUTR
        di.send_command_data(0x22, &LUT_BW)?; // LUT_R
                                              // LUTW
        di.send_command_data(0x23, &LUT_WB)?; // LUT_W
                                              // LUTB
        di.send_command_data(0x24, &LUT_BB)?; // LUT_B

        di.send_command_data(0x25, &LUT_WW)?;

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
        let n = di.send_data_from_iter(buffer)?;

        // empty red channel
        di.send_command(0x13)?;
        di.send_data_from_iter(iter::repeat(&0).take(n))?;
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x04, &[0x00])?; // Power on
        Self::busy_wait(di)?;

        di.send_command_data(0x12, &[0x00])?; // display refresh
        Self::busy_wait(di)?;

        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x02, &[0x00])?; // power off
        delay.delay_us(5_000_u32);
        Self::busy_wait(di)?;

        Ok(())
    }
}

impl MultiColorDriver for PervasiveDisplays {
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
            //
        }
        Ok(())
    }
}

impl WaveformDriver for PervasiveDisplays {
    fn update_waveform<DI: DisplayInterface>(
        _di: &mut DI,
        _lut: &'static [u8],
    ) -> Result<(), Self::Error> {
        todo!() // unused, since it has multiple LUTs
    }
}

impl FastUpdateDriver for PervasiveDisplays {
    fn setup_fast_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_VCOM: [u8; 44] = [
            // Level select, Number of Frames 0-3, Times to repeat
            // ALL off here
            0x00, 0x00, 0x00, 0x0A, 0x00, 0x00,
            0x00, 0x01, 0x60, 0x14, 0x14, 0x00,
            0x00, 0x01, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x13, 0x0A, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ST_XON[6:0], ST_CHV[6:0]
            0x00, 0x00,
        ];
        // white to white
        #[rustfmt::skip]
        const LUT_WW: [u8; 42] = [
            0b10_00_00_00, 0x0f, 0x00, 0x00, 0x00, 0x01,
            0x90, 0x00, 0x14, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x0A, 0x00, 0x00, 0x00,
            0xA0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        // black to white
        #[rustfmt::skip] // LUT_R
        const LUT_BW: [u8; 42] = [
            0b01_10_00_00, 0x01, 0x02, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        #[rustfmt::skip] // x LUT_W
        const LUT_WB: [u8; 42] = [
            // LEVEL, frame0, frame1, frame2, frame3, RP
            0b01_10_00_00, 0x01, 0x15, 0x00, 0x00, 0x01,
            0x90, 0x14, 0x14, 0x00, 0x00, 0x00,
            0x00, 0x14, 0x0A, 0x00, 0x00, 0x00,
            0x99, 0x0B, 0x04, 0x04, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        #[rustfmt::skip] // x LUT_B
        const LUT_BB: [u8; 42] = [
            0b10_01_00_00, 0x02, 0x2c, 0x00, 0x00, 0x01,
            0x90, 0x14, 0x14, 0x00, 0x00, 0x00,
            0x20, 0x14, 0x0A, 0x00, 0x00, 0x00,
            0x50, 0x13, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // LUTC
        di.send_command_data(0x20, &LUT_VCOM)?;
        // LUTWW
        di.send_command_data(0x21, &LUT_WW)?;
        // LUTR
        di.send_command_data(0x22, &LUT_BW)?; // LUT_R
                                              // LUTW
        di.send_command_data(0x23, &LUT_WB)?; // LUT_W
                                              // LUTB
        di.send_command_data(0x24, &LUT_BB)?; // LUT_B

        di.send_command_data(0x25, &LUT_WW)?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_VCOM: [u8; 44] = [
            // Level select, Number of Frames 0-3, Times to repeat
            // ALL off here
            0x00, 0x00, 0x00, 0x0A, 0x00, 0x00,
            0x00, 0x01, 0x60, 0x14, 0x14, 0x00,
            0x00, 0x01, 0x00, 0x14, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x13, 0x0A, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ST_XON[6:0], ST_CHV[6:0]
            0x00, 0x00,
        ];
        // white to white
        #[rustfmt::skip]
        const LUT_WW: [u8; 42] = [
            0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
            0b10_01_10_00, 0x01, 0x03, 0x2f, 0x00, 0x01,
            0x10, 0x00, 0x0A, 0x00, 0x00, 0x00,
            0xA0, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        // black to white
        #[rustfmt::skip] // LUT_R
        const LUT_BW: [u8; 42] = [
            0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
            0b10_10_10_10, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        #[rustfmt::skip] // x LUT_W
        const LUT_WB: [u8; 42] = [
            // LEVEL, frame0, frame1, frame2, frame3, RP
            0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
            0b10_01_10_00, 0x04, 0x01, 0x25, 0x00, 0x01,
            0x00, 0x14, 0x0A, 0x00, 0x00, 0x00,
            0x99, 0x0B, 0x04, 0x04, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        #[rustfmt::skip] // x LUT_B
        const LUT_BB: [u8; 42] = [
            0x90, 0x14, 0x14, 0x00, 0x00, 0x01,
            0b01_10_01_00, 0x05, 0x03, 0x20, 0x00, 0x01,
            0x20, 0x14, 0x0A, 0x00, 0x00, 0x00,
            0x50, 0x13, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // LUTC
        di.send_command_data(0x20, &LUT_VCOM)?;
        // LUTWW
        di.send_command_data(0x21, &LUT_WW)?;
        // LUTR
        di.send_command_data(0x22, &LUT_BW)?; // LUT_R
                                              // LUTW
        di.send_command_data(0x23, &LUT_WB)?; // LUT_W
                                              // LUTB
        di.send_command_data(0x24, &LUT_BB)?; // LUT_B

        di.send_command_data(0x25, &LUT_WW)?;
        Ok(())
    }
}
