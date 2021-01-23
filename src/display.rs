//! Driver for embedded-graphics.
//!
//! IL3895.
//!
//! The buffer has to be flushed to update the display after a group of draw calls has been completed.
//! The flush is not part of embedded-graphics API.

use core::array::FixedSizeArray;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem;

//use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use crate::interface::{DisplayError, DisplayInterface};
use embedded_graphics::{drawable::Pixel, pixelcolor::BinaryColor, prelude::*, DrawTarget};

use crate::drivers::il3895::command::Command;
use crate::drivers::il3895::lut::LUT_FULL_UPDATE;

/// Rotation of the display.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degress clockwise
    Rotate90,
    /// Rotate by 180 degress clockwise
    Rotate180,
    /// Rotate 270 degress clockwise, recommend
    Rotate270,
}

#[derive(Clone, Copy, Debug)]
pub enum Mirroring {
    None,
    Horizontal,
    Vertical,
    Origin,
}

/// Trait that defines display size information
pub trait DisplaySize {
    /// Width in pixels
    const WIDTH: usize;
    /// Height in pixels
    const HEIGHT: usize;

    type Buffer: FixedSizeArray<u8>;
}

/// For 2in13 PPD with Black, Red/Yellow and White, WIDTH=104, HEIGHT=212.
pub struct DisplaySize212x104;

impl DisplaySize for DisplaySize212x104 {
    const WIDTH: usize = 104;
    const HEIGHT: usize = 212;

    type Buffer = [u8; (Self::WIDTH / 8 + 1) * Self::HEIGHT];
}

/// For 2in13 EPD with Black and White, WIDTH=122, HEIGHT=250.
pub struct DisplaySize250x122;

impl DisplaySize for DisplaySize250x122 {
    const WIDTH: usize = 122;
    const HEIGHT: usize = 250;

    type Buffer = [u8; (Self::WIDTH / 8 + 1) * Self::HEIGHT];
}

pub struct FrameBuffer<S: DisplaySize> {
    buf: S::Buffer,
    rotation: DisplayRotation,
    mirroring: Mirroring,
    _marker: PhantomData<S>,
}

impl<S: DisplaySize> FrameBuffer<S> {
    fn new() -> Self {
        let mut buf: S::Buffer = unsafe { mem::zeroed() };
        buf.as_mut_slice().iter_mut().for_each(|b| *b = 0xff);
        Self {
            buf,
            rotation: DisplayRotation::Rotate0,
            mirroring: Mirroring::None,
            _marker: PhantomData,
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, pixel: bool) {
        let width_in_byte = S::WIDTH / 8 + (S::WIDTH % 8 != 0) as usize;

        let (width, height) = match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (S::WIDTH, S::HEIGHT),
            _ => (S::HEIGHT, S::WIDTH),
        };

        if x > width || y > height {
            return; // TODO: signal this type of error
        }

        let (mut x, mut y) = match self.rotation {
            DisplayRotation::Rotate0 => (x, y),
            DisplayRotation::Rotate90 => (S::WIDTH - y - 1, x),
            DisplayRotation::Rotate180 => (S::WIDTH - x - 1, S::HEIGHT - y - 1),
            DisplayRotation::Rotate270 => (y, S::HEIGHT - x - 1),
        };

        match self.mirroring {
            Mirroring::Horizontal => {
                x = S::WIDTH - x - 1;
            }
            Mirroring::Vertical => {
                y = S::HEIGHT - y - 1;
            }
            Mirroring::Origin => {
                x = S::WIDTH - x - 1;
                y = S::HEIGHT - y - 1;
            }
            _ => (),
        }

        if x > S::WIDTH || y > S::HEIGHT {
            return; // TODO: signal error
        }

        // For black white color
        let byte_offset = y * width_in_byte + x / 8;
        if pixel {
            self.buf.as_mut_slice()[byte_offset] &= !(0x80 >> (x % 8));
        } else {
            self.buf.as_mut_slice()[byte_offset] |= 0x80 >> (x % 8);
        }
    }
}

/// EPaperDisplay SPI display interface.
pub struct EPaperDisplay<DI, SIZE: DisplaySize> {
    di: DI,
    framebuffer: FrameBuffer<SIZE>,
}

impl<DI, SIZE> EPaperDisplay<DI, SIZE>
where
    DI: DisplayInterface,
    SIZE: DisplaySize,
{
    pub fn new(di: DI) -> Self {
        EPaperDisplay {
            di,
            framebuffer: FrameBuffer::new(),
        }
    }

    /// Consume the display interface and return
    /// the underlying peripherial driver and GPIO pins used by it
    pub fn release(self) -> DI {
        self.di
    }

    /// Set the rotation of the display.
    /// For most board setting, Rotate270 is the most convenient.
    pub fn set_rotation(&mut self, rotation: DisplayRotation) {
        self.framebuffer.rotation = rotation;
    }

    /// Set the mirroring of the display.
    /// Some display requires mirroring of pixels(can also be implemented with oppsite X/Yscan direction).
    /// This setting should be changed with rotation.
    pub fn set_mirroring(&mut self, mirror: Mirroring) {
        self.framebuffer.mirroring = mirror;
    }

    /// Do a soft reset and init the display.
    pub fn init(&mut self) -> Result<(), DisplayError> {
        // After SW reset, the IC will have Registers load with POR value,
        // VCOM register loaded with OTP, and value IC enter idle mode.
        self.di.busy_wait();
        self.di.send_command(Command::SoftReset as u8)?;
        self.di.busy_wait();

        self.di.send_command(Command::DriverOutputControl as u8)?;
        // MUX Gate lines, Gate scanning sequence and direction
        self.di
            .send_data(&[((SIZE::HEIGHT - 1) & 0xff) as u8, 0x00])?;

        self.di.send_command(Command::WriteVcomRegister as u8)?;
        self.di.send_data(&[0xa8])?;

        self.di.send_command(Command::SetDummyLinePeriod as u8)?;
        self.di.send_data(&[0x1a])?;

        self.di.send_command(Command::SetGateLineWidth as u8)?;
        self.di.send_data(&[0x08])?;

        self.di.send_command(Command::BorderWaveformControl as u8)?;
        self.di.send_data(&[0x63])?;

        // NOTE: In epd, the data entry mode is not changed
        self.di.send_command(Command::DataEntryModeSetting as u8)?;
        // A[1:0] = 11, Y increment, X increment
        // A[2]   =  0, increase X
        self.di.send_data(&[0b011])?;

        self.di.send_command(Command::WriteLutRegister as u8)?;
        self.di.send_data(&LUT_FULL_UPDATE)?;
        self.di.busy_wait();

        Ok(())
    }

    /// Set drawing window
    pub fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), DisplayError> {
        // x point must be the multiple of 8 or the last 3 bits will be ignored
        self.di
            .send_command(Command::SetRamXAddressStartEndPosition as u8)?;
        self.di.send_data(&[(x0 >> 3) as u8, (x1 >> 3) as u8])?;
        self.di
            .send_command(Command::SetRamYAddressStartEndPosition as u8)?;
        self.di.send_data(&[
            (y0 & 0xff) as u8,
            (y0 >> 8) as u8,
            (y1 & 0xff) as u8,
            (y1 >> 8) as u8,
        ])?;
        Ok(())
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), DisplayError> {
        self.di.send_command(Command::SetRamXAddressCounter as u8)?;
        self.di.send_data(&[(x >> 3) as u8])?;
        self.di.send_command(Command::SetRamYAddressCounter as u8)?;
        self.di.send_data(&[(y & 0xff) as u8, (y >> 8) as u8])?;
        Ok(())
    }

    fn turn_on_display(&mut self) -> Result<(), DisplayError> {
        self.di.send_command(Command::DisplayUpdateControl2 as u8)?;
        self.di.send_data(&[0xC4])?;

        self.di.send_command(Command::MasterActivation as u8)?;

        // TERMINATE_FRAME_READ_WRITE
        self.di.send_command(0xFF)?;

        self.di.busy_wait();
        Ok(())
    }

    /*
    pub fn clear(&mut self) -> Result<(), DisplayError> {
        self.set_window(0, 0, SIZE::WIDTH as _, SIZE::HEIGHT as _)?;
        let byte_width = (SIZE::WIDTH / 8) + (SIZE::WIDTH % 8 > 0) as usize;

        for j in 0..SIZE::HEIGHT {
            self.set_cursor(0, j as _)?;
            self.di.send_command(Command::WriteRam as u8)?;
            // 0xff white
            // 0x00 black
            for _ in 0..byte_width {
                self.di.send_data(&[0xff])?;
            }
        }
        self.turn_on_display()
    }
    */

    /// Write out data to a display.
    pub fn flush(&mut self) -> Result<(), DisplayError> {
        self.set_window(0, 0, SIZE::WIDTH as _, SIZE::HEIGHT as _)?;
        let byte_width = (SIZE::WIDTH / 8) + (SIZE::WIDTH % 8 > 0) as usize;
        for j in 0..SIZE::HEIGHT {
            self.set_cursor(0, j as _)?;
            self.di.send_command(Command::WriteRam as u8)?;
            for i in 0..byte_width {
                self.di
                    .send_data(&[self.framebuffer.buf.as_slice()[j * byte_width + i]])?;
            }
        }
        self.turn_on_display()
    }
}

impl<DI, SIZE> DrawTarget<BinaryColor> for EPaperDisplay<DI, SIZE>
where
    DI: DisplayInterface,
    SIZE: DisplaySize,
{
    type Error = core::convert::Infallible;

    /// Draw a `Pixel` that has a color defined as `BinaryColor`.
    // On => black, 0x00
    // Off => white, 0xff
    fn draw_pixel(&mut self, pixel: Pixel<BinaryColor>) -> Result<(), Self::Error> {
        let Pixel(coord, color) = pixel;

        match TryInto::<(u32, u32)>::try_into(coord) {
            Ok((x, y)) => self.framebuffer.set_pixel(x as _, y as _, color.is_on()),
            _ => (),
        }

        Ok(())
    }

    // NOTE: size() should change according to rotation.
    // Some embedded_graphics drivers do not follow this specification.
    fn size(&self) -> Size {
        match self.framebuffer.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Size::new(SIZE::WIDTH as _, SIZE::HEIGHT as _)
            }
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => {
                Size::new(SIZE::HEIGHT as _, SIZE::WIDTH as _)
            }
        }
    }

    // accelerated implementation
    fn clear(&mut self, color: BinaryColor) -> Result<(), Self::Error> {
        let fill = if color.is_on() { 0x00 } else { 0xff };
        self.framebuffer
            .buf
            .as_mut_slice()
            .iter_mut()
            .for_each(|b| *b = fill);
        Ok(())
    }
}
