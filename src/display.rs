use core::convert::TryInto;
use core::iter;

use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_graphics::{
    drawable::Pixel,
    image::Image,
    pixelcolor::{
        raw::{RawData, RawU16},
        BinaryColor, Gray2, Gray4, GrayColor,
    },
    prelude::*,
    primitives::Rectangle,
    style::{PrimitiveStyle, Styled},
    DrawTarget,
};
use embedded_hal::digital::v2::InputPin;

use crate::command::Command;
use crate::lut::LUT_FULL_UPDATE;

const WIDTH_2IN13_V1: u16 = 122;
const HEIGHT_2IN13_V1: u16 = 250;
// WIDTH dots compressed into bytes.
const BYTE_WIDTH_2IN13_V1: u8 = (WIDTH_2IN13_V1 / 8) as u8 + (WIDTH_2IN13_V1 % 8 > 0) as u8;
// Lower than 4KiB
const FRAME_BUFFER_SIZE: usize = BYTE_WIDTH_2IN13_V1 as usize * HEIGHT_2IN13_V1 as usize;

#[derive(Clone, Copy)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0 = 0x3,
    /// Rotate by 90 degress clockwise
    Rotate90,
    /// Rotate by 180 degress clockwise
    Rotate180,
    /// Rotate 270 degress clockwise
    Rotate270,
}
/// EPaperDisplay SPI display interface.
pub struct EPaperDisplay<DI, BUSY> {
    di: DI,
    busy: BUSY,
    framebuffer: [u8; FRAME_BUFFER_SIZE],
}

impl<DI, BUSY> EPaperDisplay<DI, BUSY>
where
    DI: WriteOnlyDataCommand,
    BUSY: InputPin,
{
    pub fn new(di: DI, busy: BUSY) -> Self {
        let framebuffer = [0xff; FRAME_BUFFER_SIZE];
        EPaperDisplay {
            di,
            busy,
            framebuffer,
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> (DI, BUSY) {
        (self.di, self.busy)
    }

    fn send_command(&mut self, cmd: Command) -> Result<(), DisplayError> {
        self.di.send_commands(DataFormat::U8(&[cmd as u8]))
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.di.send_data(DataFormat::U8(data))
    }

    pub fn busy_wait(&self) -> Result<(), DisplayError> {
        // LOW: idle, HIGH: busy
        while self
            .busy
            .is_high()
            .map_err(|_| DisplayError::BusWriteError)?
        {}
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), DisplayError> {
        const HEIGHT: u16 = 250;

        self.send_command(Command::DriverOutputControl)?;
        self.send_data(&[((HEIGHT - 1) & 0xff) as u8, ((HEIGHT - 1) >> 8) as u8])?;

        // BOOSTER_SOFT_START_CONTROL
        // self.di.send_commands(DataFormat::U8(&[0x0C]));
        // self.di.send_data(DataFormat::U8(&[0xD7, 0xD6, 0x9D]));

        self.send_command(Command::WriteVcomRegister)?;
        self.send_data(&[0xa8])?;

        self.send_command(Command::SetDummyLinePeriod)?;
        self.send_data(&[0x1a])?;

        // SET_GATE_TIME
        self.send_command(Command::SetGateLineWidth)?;
        self.send_data(&[0x08])?;

        self.send_command(Command::BorderWaveformControl)?;
        // A = 0b1100011
        // A[6]=1: Select FIX level Setting A[5:4] for VBD [POR]
        // A[5:4]=10: VBD level is VSL
        // A[1:0] GS transition setting for VBD
        // A[1:0]=11: ?HiZ
        self.send_data(&[0x63])?;

        self.send_command(Command::DataEntryModeSetting)?;
        // A[1:0] = 11, Y increment, X increment
        // A[2] = AM = 0, address counter is updated in the X direction.
        self.send_data(&[0x03])?;

        self.send_command(Command::WriteLutRegister)?;
        self.send_data(&LUT_FULL_UPDATE)?;
        Ok(())
    }

    /// Set drawing window
    pub fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), DisplayError> {
        // x point must be the multiple of 8 or the last 3 bits will be ignored
        self.send_command(Command::SetRamXAddressStartEndPosition)?;
        self.send_data(&[(x0 >> 3) as u8, (x1 >> 3) as u8])?;
        self.send_command(Command::SetRamYAddressStartEndPosition)?;
        self.send_data(&[
            (y0 & 0xf) as u8,
            (y0 >> 8) as u8,
            (y1 & 0xff) as u8,
            (y1 >> 8) as u8,
        ])?;
        Ok(())
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), DisplayError> {
        // x point must be the multiple of 8 or the last 3 bits will be ignored
        self.send_command(Command::SetRamXAddressCounter)?;
        self.send_data(&[(x >> 3) as u8])?;
        self.send_command(Command::SetRamYAddressCounter)?;
        self.send_data(&[(y & 0xff) as u8, (y >> 8) as u8])?;
        Ok(())
    }

    pub fn turn_on_display(&mut self) -> Result<(), DisplayError> {
        self.send_command(Command::DisplayUpdateControl2)?;
        // 0xC4 = 0b11000100
        // - Enable Clock Signal
        // - Then Enable Analog
        // - Then PATTERN DISPLAY
        self.send_data(&[0xC4])?;

        self.send_command(Command::MasterActivation)?;

        // TERMINATE_FRAME_READ_WRITE
        self.di.send_commands(DataFormat::U8(&[0xFF]))?;

        self.busy_wait()
    }

    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.set_window(0, 0, WIDTH_2IN13_V1, HEIGHT_2IN13_V1)?;
        let byte_width = (WIDTH_2IN13_V1 / 8) as u8 + (WIDTH_2IN13_V1 % 8 > 0) as u8;
        for j in 0..HEIGHT_2IN13_V1 {
            self.set_cursor(0, j)?;
            self.send_command(Command::WriteRam)?;
            // 0xff write
            // 0x00 black
            self.di.send_data(DataFormat::U8Iter(
                &mut iter::repeat(0xff).take(byte_width as usize),
            ))?;
        }
        self.turn_on_display()
    }

    pub fn display(&mut self) -> Result<(), DisplayError> {
        self.set_window(0, 0, WIDTH_2IN13_V1, HEIGHT_2IN13_V1)?;
        let byte_width = ((WIDTH_2IN13_V1 / 8) as u8 + (WIDTH_2IN13_V1 % 8 > 0) as u8) as usize;
        for j in 0..HEIGHT_2IN13_V1 as usize {
            self.set_cursor(0, j as _)?;
            self.send_command(Command::WriteRam)?;
            self.di.send_data(DataFormat::U8Iter(
                &mut self.framebuffer[j * byte_width..j * byte_width + byte_width]
                    .iter()
                    .copied(),
            ))?;
        }
        self.turn_on_display()
    }
}

impl<DI, BUSY> DrawTarget<BinaryColor> for EPaperDisplay<DI, BUSY> {
    type Error = core::convert::Infallible;

    /// Draw a `Pixel` that has a color defined as `Gray8`.
    fn draw_pixel(&mut self, pixel: Pixel<BinaryColor>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        const WIDTH: u32 = WIDTH_2IN13_V1 as _;
        const HEIGHT: u32 = HEIGHT_2IN13_V1 as _;

        if let Ok((x @ 0..=WIDTH, y @ 0..=HEIGHT)) = coord.try_into() {
            let byte_index: usize = (x / 8 + y * BYTE_WIDTH_2IN13_V1 as u32) as usize;
            let bit_index: u8 = 7 - (x % 8) as u8;
            if color.is_on() {
                self.framebuffer[byte_index] &= !(1 << bit_index);
            } else {
                self.framebuffer[byte_index] |= 1 << bit_index;
            }
        }

        Ok(())
    }

    fn size(&self) -> Size {
        Size::new(WIDTH_2IN13_V1 as _, HEIGHT_2IN13_V1 as _)
    }
}
