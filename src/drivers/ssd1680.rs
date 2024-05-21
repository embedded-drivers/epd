//! SSD1680 driver
//!
//! For:
//! - GDEY029Z94 2in9 B/W/R

// 153 bytes LUT.

use core::iter;
use embedded_hal::delay::DelayNs;

use super::{Driver, FastUpdateDriver, MultiColorDriver, WaveformDriver};
use crate::interface::{DisplayError, DisplayInterface};

/// 176 Source x 296 Gate Red/Black/White
pub struct SSD1680;

impl Driver for SSD1680 {
    type Error = DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 10_000, 10_000); // HW Reset
        Self::busy_wait(di)?;

        di.send_command(0x12)?; // swreset
        Self::busy_wait(di)?;

        di.send_command_data(0x01, &[0x27, 0x01, 0x00])?; // Driver output control

        di.send_command_data(0x11, &[0b0_11])?; // data entry mode

        di.send_command_data(0x21, &[0x00, 0x80])?; // Display update control

        // fill R frame with zeros(white)
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start
        di.send_command(0x26)?;
        di.send_data_from_iter(iter::repeat(&0).take(176 * 296 / 8))?;
        di.send_command(0x7f)?; // NOP

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
        di.send_data_from_iter(buffer)?;

        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // default
        di.send_command_data(0x22, &[0xf7])?;
        // di.send_command_data(0x22, &[0xc7])?;
        di.send_command(0x20)?;
        Self::busy_wait(di)?;

        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayNs>(
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

impl WaveformDriver for SSD1680 {
    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xc7])?;
        di.send_command(0x20)?;
        Self::busy_wait(di)?;
        Ok(())
    }
    fn update_waveform<DI: DisplayInterface>(
        di: &mut DI,
        lut: &'static [u8],
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x32, lut)?;
        Ok(())
    }
}

impl FastUpdateDriver for SSD1680 {
    fn setup_fast_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT: [u8; 153] = [
            // VS
            // 00 - VSS
            // 01 - VSH1
            // 10 - VSL
            // 11 - VSH2
            0b01_00_00_00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // B
            0b10_00_00_00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // W
            0b10_00_00_00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // R | L2 = L0
            0b10_00_00_00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // L3 = L1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // TPnA, TPnB, SRnAB, TPnC, TPnD, SRnCD, RPn
            0x7f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 6
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 11
            // FR
            0b0111_0000, 0x00, 0x00, 0x00, 0x00, 0x00,
            // XON
            0x00, 0x00, 0x00,
        ];
        Self::update_waveform(di, &LUT)?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // via https://github.com/waveshare/Pico_ePaper_Code/blob/f6af2a819d1181a1629321a3ff3aaaf0b82e0fe0/c/lib/e-Paper/EPD_2in9_V2.c#L35
        #[rustfmt::skip]
        const LUT: [u8; 159] = [
           //   0           1      2  3  4  5  6  7       8      9 10 11
            0b10000000, 0b01100110, 0, 0, 0, 0, 0, 0, 0b01000000, 0, 0, 0, // LUT 0 (black to black)
            0b00010000, 0b01100110, 0, 0, 0, 0, 0, 0, 0b00100000, 0, 0, 0, // LUT 1 (black to white)
            0b10000000, 0b01100110, 0, 0, 0, 0, 0, 0, 0b01000000, 0, 0, 0, // LUT 2 (white to black)
            0b00010000, 0b01100110, 0, 0, 0, 0, 0, 0, 0b00100000, 0, 0, 0, // LUT 3 (white to white)
            0,          0,          0, 0, 0, 0, 0, 0, 0,          0, 0, 0, // LUT 4
            //TP[A]
            //  TP[B]
            //      SR[AB]
            //          TB[C]
            //              TB[D]
            //                  SR[CD]
            //                      RP
            20, 8,  0,  0,  0,  0,  1, // Group 0
            10, 10, 0,  10, 10, 0,  1, // Group 1
            0,  0,  0,  0,  0,  0,  0, // Group 2
            0,  0,  0,  0,  0,  0,  0, // Group 3
            0,  0,  0,  0,  0,  0,  0, // Group 4
            0,  0,  0,  0,  0,  0,  0, // Group 5
            0,  0,  0,  0,  0,  0,  0, // Group 6
            0,  0,  0,  0,  0,  0,  0, // Group 7
            20, 8,  0,  1,  0,  0,  1, // Group 8
            0,  0,  0,  0,  0,  0,  1, // Group 9
            0,  0,  0,  0,  0,  0,  0, // Group 11
            0,  0,  0,  0,  0,  0,  0, // Group 12
            0x44, 0x44, 0x44, 0x44, 0x44, 0x44, // Framerates (FR[0] to FR[11])
            0, 0, 0, // Gate scan selection (XON)
            0x22, // EOPT = Normal
            0x17, // VGH  = 20V
            0x41, // VSH1 = 15 V
            0,    // VSH2 = Unknown
            0x32, // VSL  = -15 V
            0x36, // VCOM = -1.3 to -1.4 (not shown on datasheet)
        ];
        Self::update_waveform(di, &LUT[..153])?;
        Ok(())
    }
}
