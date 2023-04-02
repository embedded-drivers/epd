use crate::interface::{self, DisplayInterface};
use embedded_graphics::prelude::GrayColor;
use embedded_hal::blocking::delay::DelayUs;

pub use self::ssd1608::SSD1608;

mod ssd1608;
// TOOD: add profile support
pub trait Driver {
    type Error;

    /// Wake UP and init
    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error>;

    // also set ram pos
    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error>;

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>;

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;

    fn sleep<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        _di: &mut DI,
        _delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub trait MultiColorDriver: Driver {
    fn update_channel_frame<'a, DI: DisplayInterface, I>(
        di: &mut DI,
        channel: u8,
        buffer: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>;
}

pub trait GrayScaleDriver<Color: GrayColor>: Driver {
    const LUT_FULL_UPDATE: &'static [u8];
    const LUT_FRAME_UPDATE: &'static [u8];
}

/// Red/Black/White. 400 source outputs, 300 gate outputs
/// or Red/Black. 400 source outputs, 300 gate outputs
pub struct SSD1619A;

impl Driver for SSD1619A {
    type Error = interface::DisplayError;

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 200_000, 200_000);
        di.busy_wait();

        di.send_command(0x12)?; //swreset
        di.busy_wait();

        // Set analogue then digital block control
        di.send_command_data(0x74, &[0x54])?;
        di.send_command_data(0x7e, &[0x3b])?;

        di.send_command_data(0x2b, &[0x03, 0x63])?; // reduce glitch under ACVCOM

        di.send_command_data(0x0c, &[0x8b, 0x9c, 0x96, 0x0f])?; // soft start setting

        di.send_command_data(0x01, &[0x2b, 0x01, 0x00])?; // Driver Output Control - set mux as 300

        di.send_command_data(0x11, &[0b11])?; // data entry mode, X inc, Y inc

        // 0x44, 0x45, ram x,y start,end
        di.send_command_data(0x03, &[0x20])?; // Gate Driving Voltage Control

        // A[7:0] = 41h [POR], VSH1 at 15V
        // B[7:0] = A8h [POR], VSH2 at 5V.
        // C[7:0] = 32h [POR], VSL at -15V
        //di.send_command_data(0x04, &[0x4b, 0xce, 0x3a]); // Source Driving Voltage Control

        //di.send_command_data(0x3A, &[0x21]); // dummy line, 0 to 127
        //di.send_command_data(0x3B, &[0x06]); // gate width

        // 0b10_00_00 , VCOM, black
        // 0b11_00_00, HiZ
        // 0b01_00_00, VSS
        di.send_command_data(0x3C, &[0x01])?; // border wavefrom, HIZ

        di.send_command_data(0x18, &[0x80])?;
        // load temperature and waveform setting.
        di.send_command_data(0x22, &[0xb9])?; // B1 or B9

        di.send_command(0x20)?;
        di.busy_wait();

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
        di.send_data_from_iter(buffer)?;

        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x22, &[0xf7])?; // Display Update Control 2
        di.send_command(0x20)?; // master activation
        di.busy_wait();
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
        di.send_command_data(0x4e, &[0])?; // x start
        di.send_command_data(0x4f, &[0, 0])?; // y start

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
