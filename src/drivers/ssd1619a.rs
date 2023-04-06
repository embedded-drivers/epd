//! SSD1619A driver in B/W or B/W/R mode.

/*
R  B/W  LUT
0  0    0    B
0  1    1    W
1  0    2    R
1  1    3    R

LUT4: VCOM
 */

use core::iter;

use crate::interface::{self, DisplayInterface};
use embedded_graphics::pixelcolor::Gray4;
use embedded_hal::blocking::delay::DelayUs;

use super::{Driver, FastUpdateDriver, GrayScaleDriver, MultiColorDriver, WaveformDriver};

/// Red/Black/White. 400 source outputs, 300 gate outputs,
/// or Red/Black. 400 source outputs, 300 gate outputs.
/// 70 bytes LUT table.
pub struct SSD1619A;

impl Driver for SSD1619A {
    type Error = interface::DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 200_000, 200_000);
        Self::busy_wait(di)?;

        di.send_command(0x12)?; //swreset
        Self::busy_wait(di)?;

        // Set analogue then digital block control
        di.send_command_data(0x74, &[0x54])?;
        di.send_command_data(0x7e, &[0x3b])?;

        di.send_command_data(0x2b, &[0x03, 0x63])?; // reduce glitch under ACVCOM

        di.send_command_data(0x0c, &[0x8b, 0x9c, 0x96, 0x0f])?; // soft start setting

        di.send_command_data(0x01, &[0x2b, 0x01, 0x00])?; // Driver Output Control - set mux as 300

        di.send_command_data(0x11, &[0b11])?; // data entry mode, X inc, Y inc

        // 0x44, 0x45, ram x,y start,end
        // di.send_command_data(0x03, &[0x20])?; // Gate Driving Voltage Control
        // A[7:0] = 41h [POR], VSH1 at 15V
        // B[7:0] = A8h [POR], VSH2 at 5V.
        // C[7:0] = 32h [POR], VSL at -15V
        //di.send_command_data(0x04, &[0x4b, 0xce, 0x3a]); // Source Driving Voltage Control
        //di.send_command_data(0x3A, &[0x21]); // dummy line, 0 to 127
        //di.send_command_data(0x3B, &[0x06]); // gate width

        // 0b10_00_00, VCOM, black
        // 0b11_00_00, HiZ
        // 0b01_00_00, VSS
        di.send_command_data(0x3C, &[0x01])?; // border wavefrom, HIZ

        // use internal temp sensor
        di.send_command_data(0x18, &[0x80])?;
        // load temperature and waveform setting.
        di.send_command_data(0x22, &[0xb9])?; // B1 or B9
                                              // master activation
        di.send_command(0x20)?;
        Self::busy_wait(di)?;

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
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

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

        // fill R frame with zeros(white)
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

        di.send_command(0x26)?;
        di.send_data_from_iter(iter::repeat(&0).take(n))?;

        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // 0xf7: always use in system LUT
        di.send_command_data(0x22, &[0xf7])?;
        di.send_command(0x20)?; // master activation
        Self::busy_wait(di)?;
        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        _delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x10, &[0x01])?; // or 0x02 for deep sleep mode 2

        // will be busy forever
        Ok(())
    }
}

impl MultiColorDriver for SSD1619A {
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

impl WaveformDriver for SSD1619A {
    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // 0xf7: always use in system LUT
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

impl GrayScaleDriver<Gray4> for SSD1619A {
    fn setup_gray_scale_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_INCREMENTAL_DIV_16: [u8; 70] = [
            // VS
            // 00 – VSS
            // 01 – VSH1
            // 10 – VSL
            // 11 – VSH2
            0b01_00_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L0 => B
            0b00_00_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L1 => W
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L4
            // TP0                  RP[0]
            0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        Self::update_waveform(di, &LUT_INCREMENTAL_DIV_16)?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_FAST_UPDATE: [u8; 70] = [
            0b10_10_01_01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L0 => B
            0b10_01_10_10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L1 => W
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L4
            // TP0                  RP[0]
            0x30, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        Self::update_waveform(di, &LUT_FAST_UPDATE)?;

        Ok(())
    }
}

impl FastUpdateDriver for SSD1619A {
    fn setup_fast_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT_FAST: [u8; 70] = [
            // VS
            // 00 – VSS
            // 01 – VSH1
            // 10 – VSL
            // 11 – VSH2
            0b10_01_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L0 => B
            0b01_10_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L1 => W
            0b00_00_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L2 => B
            0b00_00_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L3 => W
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L4
            // TP0                  RP[0]
            0x1f, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        Self::update_waveform(di, &LUT_FAST)?;

        // gate level: VGH
        di.send_command_data(0x03, &[0x19])?; // POR, ok

        // source level: VSH1, VSH2, VSL
        di.send_command_data(0x04, &[0x4b, 0xa8, 0x32])?;
        // dummy line
        di.send_command_data(0x3a, &[0x1a])?;
        // gate line
        di.send_command_data(0x3b, &[0x0b])?;

        // VCOM
        // di.send_command_data(0x2c, &[0x78])?;

        Self::busy_wait(di)?;
        Ok(())
    }

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        #[rustfmt::skip]
        const LUT:[u8; 70] = [
            0b10_10_10_10, 0b01_01_01_01, 0b01_00_00_00, 0x00, 0x00, 0x00, 0x00, // L0 => B
            0b10_10_10_10, 0b01_01_01_01, 0b10_00_00_00, 0x00, 0x00, 0x00, 0x00, // L1 => W
            0b00_00_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L2 => B
            0b00_00_00_00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L3 => W
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // L4
            // TP0                  RP[0]
            0x0f, 0x00, 0x00, 0x00, 0x00,
            0x0f, 0x00, 0x00, 0x00, 0x00,
            0x1f, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
            ];
        Self::update_waveform(di, &LUT[..])?;
        Ok(())
    }
}
