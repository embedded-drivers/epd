//! Driver for embedded-graphics.
//!
//! IL3895.
//!
//! The buffer has to be flushed to update the display after a group of draw calls has been completed.
//! The flush is not part of embedded-graphics API.

use core::convert::TryInto;
use core::mem;

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{BinaryColor, Gray2, Gray4, Gray8},
    prelude::*,
    primitives::Rectangle,
};

use crate::color::GrayColorInBits;

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

/// 2in9
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

#[derive(Clone, Copy)]
pub struct DisplaySize104x201;

impl DisplaySize for DisplaySize104x201 {
    const WIDTH: usize = 212;
    const HEIGHT: usize = 104;

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
pub struct FrameBuffer<SIZE: DisplaySize>
where
    [(); SIZE::N]:,
{
    buf: [u8; SIZE::N],
    rotation: DisplayRotation,
    mirroring: Mirroring,
    inverted: bool,
}

impl<SIZE: DisplaySize> FrameBuffer<SIZE>
where
    [(); SIZE::N]:,
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
        let mut this = Self::new();
        this.buf.fill(0xff);
        this.inverted = true;
        this
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
            defmt::warn!("overflow set {},{}  {}", x, y, pixel);

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
            self.buf.as_mut_slice()[byte_offset] &= !(0x80 >> (x % 8));
        } else {
            self.buf.as_mut_slice()[byte_offset] |= 0x80 >> (x % 8);
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    fn size(&self) -> Size {
        Size::new(SIZE::WIDTH as _, SIZE::HEIGHT as _)
    }
}

impl<SIZE: DisplaySize> Dimensions for FrameBuffer<SIZE>
where
    [(); SIZE::N]:,
{
    fn bounding_box(&self) -> Rectangle {
        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => Rectangle::new(
                Point::zero(),
                Size::new(SIZE::WIDTH as _, SIZE::HEIGHT as _),
            ),
            _ => Rectangle::new(
                Point::zero(),
                Size::new(SIZE::HEIGHT as _, SIZE::WIDTH as _),
            ),
        }
    }
}

impl<SIZE: DisplaySize> DrawTarget for FrameBuffer<SIZE>
where
    [(); SIZE::N]:,
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

#[derive(Clone)]
pub struct GrayFrameBuffer<SIZE: DisplaySize, C: GrayColor + GrayColorInBits>
where
    [(); SIZE::N]:,
    [(); SIZE::N * C::BITS_PER_PIXEL]:,
{
    buf: [u8; SIZE::N * C::BITS_PER_PIXEL],
    rotation: DisplayRotation,
    mirroring: Mirroring,
}

impl<SIZE: DisplaySize, C: GrayColor + GrayColorInBits> GrayFrameBuffer<SIZE, C>
where
    [(); SIZE::N]:,
    [(); SIZE::N * C::BITS_PER_PIXEL]:,
{
    pub fn new() -> Self {
        let mut buf: [u8; SIZE::N * C::BITS_PER_PIXEL] = unsafe { mem::zeroed() };
        buf.fill(0xff);

        Self {
            buf,
            rotation: DisplayRotation::Rotate0,
            mirroring: Mirroring::None,
        }
    }

    pub fn fill(&mut self, color: BinaryColor) {
        if color.is_on() {
            self.buf.fill(0xff);
        } else {
            self.buf.fill(0x00);
        }
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

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    pub(crate) fn get_pixel_in_raw_pos(&self, x: usize, y: usize) -> C {
        if x >= SIZE::WIDTH || y >= SIZE::HEIGHT {
            return C::WHITE;
        }
        let width_in_bits = SIZE::WIDTH * C::BITS_PER_PIXEL;
        let width_in_byte = width_in_bits / 8 + (width_in_bits % 8 != 0) as usize;

        let mut luma = 0;
        for i in 0..C::BITS_PER_PIXEL {
            let bit_offset = x * C::BITS_PER_PIXEL + i;
            let byte_offset = width_in_byte * y + bit_offset / 8;
            let bit_offset = 7 - bit_offset % 8;

            let bit = self.buf[byte_offset] & (1 << bit_offset) != 0;
            if bit {
                luma |= 1 << i;
            }
        }
        C::from_u8(luma)
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: C) {
        let (width, height) = match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (SIZE::WIDTH, SIZE::HEIGHT),
            _ => (SIZE::HEIGHT, SIZE::WIDTH),
        };

        if x >= width || y >= height {
            defmt::warn!("overflow set {},{}  {}", x, y, pixel.luma());
            return;
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

        let width_in_bits = SIZE::WIDTH * C::BITS_PER_PIXEL;
        let width_in_byte = width_in_bits / 8 + (width_in_bits % 8 != 0) as usize;

        for i in 0..C::BITS_PER_PIXEL {
            let bit_offset = x * C::BITS_PER_PIXEL + i;
            let byte_offset = width_in_byte * y + bit_offset / 8;
            let bit_offset = 7 - bit_offset % 8;

            if pixel.luma() & (1 << i) != 0 {
                self.buf.as_mut_slice()[byte_offset] |= 1 << bit_offset;
            } else {
                self.buf.as_mut_slice()[byte_offset] &= !(1 << bit_offset);
            }
        }
    }

    pub fn bounding_box(&self) -> Rectangle {
        match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => Rectangle::new(
                Point::zero(),
                Size::new(SIZE::WIDTH as _, SIZE::HEIGHT as _),
            ),
            _ => Rectangle::new(
                Point::zero(),
                Size::new(SIZE::HEIGHT as _, SIZE::WIDTH as _),
            ),
        }
    }
}
