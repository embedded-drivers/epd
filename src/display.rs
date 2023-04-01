//! Driver for embedded-graphics.
//!
//! IL3895.
//!
//! The buffer has to be flushed to update the display after a group of draw calls has been completed.
//! The flush is not part of embedded-graphics API.

use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem;

//use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use crate::interface::{DisplayError, DisplayInterface};
use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};

// use crate::drivers::il3895::command::Command;
// use crate::drivers::il3895::lut::LUT_FULL_UPDATE;

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

    const N: usize;
}

#[derive(Clone, Copy)]
pub struct DisplaySize128x296;

impl DisplaySize for DisplaySize128x296 {
    const WIDTH: usize = 128;
    const HEIGHT: usize = 296;

    const N: usize = (Self::WIDTH / 8) * Self::HEIGHT;
}

/// SSD1608/IL3820 in cascade mode 2x 200x300
#[derive(Clone, Copy)]
pub struct DisplaySize200x300;

impl DisplaySize for DisplaySize200x300 {
    const WIDTH: usize = 200;
    const HEIGHT: usize = 300;

    const N: usize = (Self::WIDTH / 8) * Self::HEIGHT;
}

/// For 2in13 PPD with Black, Red/Yellow and White, WIDTH=104, HEIGHT=212.
#[derive(Clone, Copy)]
pub struct DisplaySize212x104;

impl DisplaySize for DisplaySize212x104 {
    const WIDTH: usize = 104;
    const HEIGHT: usize = 212;

    const N: usize = (Self::WIDTH / 8 + 1) * Self::HEIGHT;
}

/// For 2in13 EPD with Black and White, WIDTH=122, HEIGHT=250.
#[derive(Clone, Copy)]
pub struct DisplaySize250x122;

impl DisplaySize for DisplaySize250x122 {
    const WIDTH: usize = 122;
    const HEIGHT: usize = 250;

    const N: usize = (Self::WIDTH / 8 + 1) * Self::HEIGHT;
}

// 4in2
#[derive(Clone, Copy)]
pub struct DisplaySize400x300;

impl DisplaySize for DisplaySize400x300 {
    const WIDTH: usize = 400;
    const HEIGHT: usize = 300;

    const N: usize = (Self::WIDTH / 8) * Self::HEIGHT;
}

// TODO: active ON/OFF pixel
#[derive(Clone)]
pub struct FrameBuffer<SIZE: DisplaySize> {
    buf: [u8; SIZE::N],
    rotation: DisplayRotation,
    mirroring: Mirroring,
    inverted: bool,
}

impl<SIZE: DisplaySize> FrameBuffer<SIZE>
{
    pub fn new() -> Self {
        let buf = unsafe { mem::zeroed() };

        Self {
            buf,
            rotation: DisplayRotation::Rotate0,
            mirroring: Mirroring::None,
            inverted: false,
        }
    }

    pub fn new_inverted() -> Self {
        let mut buf: [u8; (SIZE::WIDTH / 8 + (SIZE::WIDTH % 8 != 0) as usize) * SIZE::HEIGHT] =
            unsafe { mem::zeroed() };
        buf.fill(0xff);

        Self {
            buf,
            rotation: DisplayRotation::Rotate0,
            mirroring: Mirroring::None,
            inverted: true,
        }
    }

    pub fn fill(&mut self, color: BinaryColor) {
        let color_raw = match (color, self.inverted) {
            (BinaryColor::On, true) | (BinaryColor::Off, false) => 0xff,
            (BinaryColor::Off, true) | (BinaryColor::On, false) => 0x00,
        };
        self.buf.fill(color_raw)
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.rotation = match rotation {
            0 => DisplayRotation::Rotate0,
            90 => DisplayRotation::Rotate90,
            180 => DisplayRotation::Rotate180,
            270 => DisplayRotation::Rotate270,
            _ => DisplayRotation::Rotate0,
        };
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    pub fn set_inverted(&mut self, inverted: bool) {
        self.inverted = inverted;
        self.buf.iter_mut().for_each(|b| *b = !*b);
    }

    fn set_pixel(&mut self, x: usize, y: usize, pixel: bool) {
        let width_in_byte = SIZE::WIDTH / 8 + (SIZE::WIDTH % 8 != 0) as usize;

        let (width, height) = match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (SIZE::WIDTH, SIZE::HEIGHT),
            _ => (SIZE::HEIGHT, SIZE::WIDTH),
        };

        if x > width || y > height {
            defmt::error!("overflow set {},{}  {}", x, y, pixel);

            return; // TODO: signal this type of error
        }

        let (mut x, mut y) = match self.rotation {
            DisplayRotation::Rotate0 => (x, y),
            DisplayRotation::Rotate90 => (SIZE::WIDTH - y - 1, x),
            DisplayRotation::Rotate180 => (SIZE::WIDTH - x - 1, SIZE::HEIGHT - y - 1),
            DisplayRotation::Rotate270 => (y, SIZE::HEIGHT - x - 1),
        };

        match self.mirroring {
            Mirroring::Horizontal => {
                x = SIZE::WIDTH - x - 1;
            }
            Mirroring::Vertical => {
                y = SIZE::HEIGHT - y - 1;
            }
            Mirroring::Origin => {
                x = SIZE::WIDTH - x - 1;
                y = SIZE::HEIGHT - y - 1;
            }
            _ => (),
        }

        if x > SIZE::WIDTH || y > SIZE::HEIGHT {
            defmt::error!("set {},{}  {}", x, y, pixel);

            return; // TODO: signal error
        }

        // For black white color
        let byte_offset = y * width_in_byte + x / 8;
        if pixel ^ self.inverted {
            self.buf.as_mut_slice()[byte_offset] |= 0x80 >> (x % 8);
        } else {
            self.buf.as_mut_slice()[byte_offset] &= !(0x80 >> (x % 8));
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    fn size(&self) -> Size {
        Size::new(SIZE::WIDTH as _, SIZE::HEIGHT as _)
    }
}

/*
impl<S: DisplaySize> OriginDimensions for FrameBuffer<S, { S::N }> {
    fn size(&self) -> Size {
        Size::new(S::WIDTH as _, S::HEIGHT as _)
    }
}
*/

impl<SIZE: DisplaySize> Dimensions for FrameBuffer<SIZE>
{
    fn bounding_box(&self) -> Rectangle {
        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Rectangle::new(Point::zero(), Size::new(SIZE::WIDTH as _, SIZE::HEIGHT as _))
            }
            _ => Rectangle::new(Point::zero(), Size::new(SIZE::HEIGHT as _, SIZE::WIDTH as _)),
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> DrawTarget for FrameBuffer<WIDTH, HEIGHT>
where
    [(); (WIDTH / 8 + (WIDTH % 8 != 0) as usize) * HEIGHT]:,
{
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            match TryInto::<(u32, u32)>::try_into(coord) {
                Ok((x, y)) => self.set_pixel(x as _, y as _, color.is_on()),
                _ => (),
            }
        }

        Ok(())
    }
}
/*
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
*/
