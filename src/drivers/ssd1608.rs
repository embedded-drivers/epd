use embedded_graphics::pixelcolor::{Gray2, Gray4};
use embedded_hal::blocking::delay::DelayUs;

use crate::{
    color::Gray3,
    interface::{self, DisplayInterface},
};

use super::{Driver, GrayScaleDriver, WaveformDriver};

/// B/W 240 x 320
pub struct SSD1608;

impl Driver for SSD1608 {
    type Error = interface::DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 200_000, 200_000);
        Self::busy_wait(di)?;

        defmt::debug!("wake up");

        // TODO: deep sleep?
        // di.send_command_data(0x10, &[0x00])?;

        di.send_command(0x12)?; //swreset
        Self::busy_wait(di)?;

        // Booster Enable with Phase 1, Phase 2 and Phase 3 for soft start current setting.
        di.send_command_data(0x0c, &[0xd7, 0xd6, 0x9d])?;

        // write VCOM reg
        di.send_command_data(0x2c, &[0x7c])?; //a8

        // Set dummy line period
        di.send_command_data(0x3a, &[0x1a])?;
        // Set Gate line width
        di.send_command_data(0x3b, &[0x08])?;

        // optional voltage control
        //di.send_command_data(0x04, &[0b0000])?;

        // Border Waveform Control
        // 00 VSS => no change
        // 01 VSH => very black
        // 10 VSL => white
        // 11 HiZ => no change
        di.send_command_data(0x3c, &[0b1_1_10_00_00])?; // border waveform control

        // Data Entry mode,
        // Y increment, X increment
        // address counter is updated in the X direction. [POR]
        di.send_command_data(0x11, &[0x03])?;

        // https://github.com/TeXitoi/il3820/blob/master/src/lib.rs
        #[rustfmt::skip]
        const EPD_2_IN13_LUT_FULL_UPDATE: [u8; 30] = [
            0x50, 0xAA, 0x55, 0xAA, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,

            0xFF, 0xFF, 0x1F, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00,
        ];
        #[rustfmt::skip]
        const EPD_2_IN13_LUT_PARTIAL_UPDATE: [u8; 30] = [
            //0x22, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x11,
            //0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            //0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E,
            //0x01, 0x00, 0x00, 0x00, 0x00, 0x00
            // VS
            // fast update
            0b10_01_10_01,
            // 0x22,
            0x00,
                        0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            // TP
            0x0a, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00
        ];

        di.send_command_data(0x32, &EPD_2_IN13_LUT_PARTIAL_UPDATE)?;

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        // Driver Output control
        di.send_command_data(0x01, &[((y - 1) & 0xff) as u8, ((y - 1) >> 8) as u8, 0])?;

        // set ram x start/end
        di.send_command_data(0x44, &[0, ((x - 1) >> 3) as u8])?;
        // set ram y start/end
        di.send_command_data(0x45, &[0, 0, ((y - 1) & 0xff) as u8, ((y - 1) >> 8) as u8])?;
        Ok(())
    }

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        // set cursor
        di.send_command_data(0x4E, &[0])?;
        di.send_command_data(0x4f, &[0, 0])?;

        // write ram
        di.send_command(0x24)?;
        di.send_data_from_iter(buffer)?;

        di.send_command(0xff)?;
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xc4])?; // Display Update Control 2
        di.send_command(0x20)?;
        di.send_command(0xff)?;
        Self::busy_wait(di)?;
        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        _delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x10, &[0x01])?;
        Ok(())
    }
}

/// Fast update driver for SSD1608
pub struct SSD1608Fast;

impl Driver for SSD1608Fast {
    type Error = interface::DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_FAST_UPDATE: [u8; 30] = [
            // VS
            // fast update
            0b10_01_10_01,
            /**/  0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            // TP
            0x0a, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            //  VSH/VSL and Dummy bit
            0x00, 0x00
        ];
        SSD1608::wake_up(di, delay)?;
        di.send_command_data(0x32, &LUT_FAST_UPDATE)?;
        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        SSD1608::set_shape(di, x, y)
    }

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        SSD1608::update_frame(di, buffer)
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        <SSD1608 as Driver>::turn_on_display(di)
    }
}

impl WaveformDriver for SSD1608 {
    fn update_waveform<DI: DisplayInterface>(
        di: &mut DI,
        lut: &'static [u8],
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x32, lut)
    }
}

impl GrayScaleDriver<Gray2> for SSD1608 {
    fn setup_gray_scale_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_INCREMENTAL_DIV_2: [u8; 30] = [
            // VS
            // incremental update
            0b00_01_00_01,
                  0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            // TP
            0x03, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00
        ];

        Self::update_waveform(di, &LUT_INCREMENTAL_DIV_2)?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_FULL_UPDATE: [u8; 30] = [
            0x50, 0xAA, 0x55, 0xAA, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,

            0xFF, 0xFF, 0x1F, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00,
        ];
        Self::update_waveform(di, &LUT_FULL_UPDATE)?;
        Ok(())
    }
}

impl GrayScaleDriver<Gray3> for SSD1608 {
    fn setup_gray_scale_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_INCREMENTAL_DIV_16: [u8; 30] = [
            // VS
            // incremental update
            0b00_01_00_01,
                  0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            // TP
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00
        ];

        di.send_command_data(0x04, &[0b0000])?; // lower VSH/VSL

        Self::update_waveform(di, &LUT_INCREMENTAL_DIV_16)?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_FULL_UPDATE: [u8; 30] = [
            0x50, 0xAA, 0x55, 0xAA, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,

            0xFF, 0xFF, 0x1F, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00,
        ];
        Self::update_waveform(di, &LUT_FULL_UPDATE)?;
        Ok(())
    }
}

impl GrayScaleDriver<Gray4> for SSD1608 {
    fn setup_gray_scale_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_INCREMENTAL_DIV_16: [u8; 30] = [
            // VS
            // incremental update
            // 10 clean
            0b00_01_00_01,
                  0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            // TP
            0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00
        ];

        // write VCOM reg
        di.send_command_data(0x2c, &[0xb8])?; // Good to distinguish between gray levels

        // di.send_command_data(0x03, &[0b0000_0000])?; // VGH/VGL
        di.send_command_data(0x04, &[0b0000])?; // lower VSH/VSL
        di.send_command_data(0x3b, &[0b0000])?; // lowest gate line width

        Self::update_waveform(di, &LUT_INCREMENTAL_DIV_16)?;

        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_FULL_UPDATE: [u8; 30] = [
            0x50, 0xAA, 0x55, 0xAA, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,

            0xFF, 0xFF, 0x1F, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00,
        ];
        Self::update_waveform(di, &LUT_FULL_UPDATE)?;
        Ok(())
    }
}
