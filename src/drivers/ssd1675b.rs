//! SSD1675B driver

use core::iter;
use embedded_hal::blocking::delay::DelayUs;

use super::{Driver, FastUpdateDriver, MultiColorDriver, WaveformDriver};
use crate::interface::{DisplayError, DisplayInterface};

/// 160 Source x 296 Gate Red/Black/White.
/// 100 bytes LUT. almost the same as SSD1619A.
pub struct SSD1675B;

impl Driver for SSD1675B {
    type Error = DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 200_000, 200_000);
        Self::busy_wait(di)?;

        di.send_command(0x12)?; //swreset
        Self::busy_wait(di)?;

        di.send_command_data(0x74, &[0x54])?;
        di.send_command_data(0x7e, &[0x3b])?;

        di.send_command_data(0x2b, &[0x03, 0x63])?; // reduce glitch under ACVCOM

        di.send_command_data(0x0c, &[0x8b, 0x9c, 0x96, 0x0f])?; // soft start setting

        di.send_command_data(0x01, &[0x2b, 0x01, 0x00])?; // Driver Output Control - set mux as 300

        di.send_command_data(0x11, &[0b11])?; // data entry mode, X inc, Y inc

        di.send_command_data(0x3C, &[0x01])?; // border wavefrom, HIZ

        // use internal temp sensor
        di.send_command_data(0x18, &[0x80])?;
        // load temperature and waveform setting.
        di.send_command_data(0x22, &[0xb9])?; // B1 or B9
                                              // master activation
        di.send_command(0x20)?;
        Self::busy_wait(di)?;

        // fill R frame with zeros(white)
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start
        di.send_command(0x26)?;
        di.send_data_from_iter(iter::repeat(&0).take(160 * 296 / 8))?;

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
        // 0xf7: always use in system LUT
        di.send_command_data(0x22, &[0xf7])?;
        di.send_command(0x20)?; // master activation
        Self::busy_wait(di)?;
        Ok(())
    }
}

impl MultiColorDriver for SSD1675B {
    fn update_channel_frame<'a, DI: DisplayInterface, I>(
        di: &mut DI,
        channel: u8,
        buffer: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        // s start and y start
        di.send_command_data(0x4e, &[0])?;
        di.send_command_data(0x4f, &[0, 0])?;

        if channel == 0 {
            di.send_command(0x24)?;
            di.send_data_from_iter(buffer)?;
        } else if channel == 1 {
            di.send_command(0x26)?;
            di.send_data_from_iter(buffer)?;
        } else {
            // error
        }

        Ok(())
    }
}

impl WaveformDriver for SSD1675B {
    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xc5])?;
        di.send_command(0x20)?;
        Self::busy_wait(di)?;
        Ok(())
    }
    fn update_waveform<DI: DisplayInterface>(
        di: &mut DI,
        lut: &'static [u8],
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x32, lut)
    }
}


// TODO: test this
impl FastUpdateDriver for SSD1675B {
    fn setup_fast_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT: [u8; 105] = [
            // VS
            0x2A, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //1
            0x05, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //2
            0x2A, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //3
            0x05, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //4
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //5

            0x00, 0x02, 0x03, 0x0A, 0x00, 0x02, 0x06, 0x0A, 0x05, 0x00, //6
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //7
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //9
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //10
            0x22, 0x22, 0x22, 0x22, 0x22,
        ];

        Self::update_waveform(di, &LUT[..])?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT: [u8; 105] = [
            // VS
            0x2A, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //1
            0x05, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //2
            0x2A, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //3
            0x05, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //4
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //5

            0x00, 0x02, 0x03, 0x0A, 0x00, 0x02, 0x06, 0x0A, 0x05, 0x00, //6
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //7
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //9
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //10
            0x22, 0x22, 0x22, 0x22, 0x22,
        ];
        Self::update_waveform(di, &LUT[..])?;
        Ok(())
    }
}
