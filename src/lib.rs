#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_arg_infer)]

pub mod display;
// pub mod drivers;
pub mod interface;

use core::{marker::PhantomData, mem};

use display::{DisplayRotation, DisplaySize, FrameBuffer, GrayColorInBits};
use drivers::{Driver, GrayScaleDriver, MultiColorDriver};
use embedded_graphics::{
    image::ImageRaw,
    pixelcolor::{BinaryColor, Gray2},
    prelude::{Dimensions, DrawTarget, GrayColor, PixelColor},
    primitives::Rectangle,
    Pixel,
};
use interface::DisplayInterface;
pub use interface::EPDInterface;

pub mod drivers;

pub struct EPD<I: DisplayInterface, S: DisplaySize, D: Driver>
where
    [(); S::N]:,
{
    pub interface: I,
    pub framebuf: FrameBuffer<S>,
    _phantom: PhantomData<(S, D)>,
}

impl<DI: DisplayInterface, S: DisplaySize, D: Driver> EPD<DI, S, D>
where
    [(); S::N]:,
{
    pub fn new(interface: DI) -> Self {
        Self {
            interface,
            framebuf: FrameBuffer::new_inverted(),
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::update_frame(&mut self.interface, self.framebuf.as_bytes())?;
        D::turn_on_display(&mut self.interface)
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn wake_up<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> Dimensions for EPD<I, S, D>
where
    [(); S::N]:,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf.bounding_box()
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> DrawTarget for EPD<I, S, D>
where
    [(); S::N]:,
{
    type Color = embedded_graphics::pixelcolor::BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<IP>(&mut self, pixels: IP) -> Result<(), Self::Error>
    where
        IP: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.framebuf.draw_iter(pixels)
    }
}

pub struct TriColorEPD<I: DisplayInterface, S: DisplaySize, D: Driver>
where
    [(); S::N]:,
{
    pub interface: I,
    pub framebuf0: FrameBuffer<S>,
    pub framebuf1: FrameBuffer<S>,
    _phantom: PhantomData<(S, D)>,
}

impl<DI: DisplayInterface, S: DisplaySize, D: MultiColorDriver> TriColorEPD<DI, S, D>
where
    [(); S::N]:,
{
    pub fn new(interface: DI) -> Self {
        Self {
            interface,
            framebuf0: FrameBuffer::new_inverted(),
            framebuf1: FrameBuffer::new(),
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::wake_up(&mut self.interface, delay)?;

        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;

        Ok(())
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::update_channel_frame(&mut self.interface, 0, self.framebuf0.as_bytes())?;
        D::update_channel_frame(&mut self.interface, 1, self.framebuf1.as_bytes())?;
        D::turn_on_display(&mut self.interface)
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn wake_up<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> Dimensions for TriColorEPD<I, S, D>
where
    [(); S::N]:,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf0.bounding_box()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TriColor {
    White,
    Black,
    Red, // or yellow?
}
/// The `Raw` can be is set to `()` because `EpdColor` doesn't need to be
/// converted to raw data for the display and isn't stored in images.
impl PixelColor for TriColor {
    type Raw = ();
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> DrawTarget for TriColorEPD<I, S, D>
where
    [(); S::N]:,
{
    type Color = TriColor;
    type Error = core::convert::Infallible;

    fn draw_iter<IP>(&mut self, pixels: IP) -> Result<(), Self::Error>
    where
        IP: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            match color {
                TriColor::White => {
                    self.framebuf0.draw_iter([Pixel(point, BinaryColor::On)])?;
                    self.framebuf1.draw_iter([Pixel(point, BinaryColor::Off)])?;
                }
                TriColor::Black => {
                    self.framebuf0.draw_iter([Pixel(point, BinaryColor::Off)])?;
                    self.framebuf1.draw_iter([Pixel(point, BinaryColor::Off)])?;
                }
                TriColor::Red => {
                    self.framebuf0.draw_iter([Pixel(point, BinaryColor::On)])?;
                    self.framebuf1.draw_iter([Pixel(point, BinaryColor::On)])?;
                }
            }
        }
        Ok(())
    }
}

pub struct GrayScaleEPD<'a, C, I: DisplayInterface, S: DisplaySize, D: GrayScaleDriver<C>>
where
    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
    [(); S::N]:,
    [(); C::BITS_PER_PIXEL]:,
{
    pub interface: I,
    pub framebufs: [FrameBuffer<S>; C::BITS_PER_PIXEL],
    _phantom: PhantomData<(S, D)>,
}

impl<'a, C, I: DisplayInterface, S: DisplaySize, D: GrayScaleDriver<C>> GrayScaleEPD<'a, C, I, S, D>
where
    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
    [(); S::N]:,
    [(); C::BITS_PER_PIXEL]:,
{
    pub fn new(interface: I) -> Self {
        let mut fbs: [FrameBuffer<S>; C::BITS_PER_PIXEL] = unsafe { mem::zeroed() };
        for fb in fbs.iter_mut() {
            fb.set_inverted(true);
        }
        Self {
            interface,
            // TODO: refactor using compressed framebuffers
            framebufs: fbs,
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;

        Ok(())
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        for fb in self.framebufs.iter_mut() {
            fb.set_rotation(rotation);
        }
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::setup_gray_scale(&mut self.interface)?;
        for fb in self.framebufs.iter() {
            D::update_frame(&mut self.interface, fb.as_bytes())?;
            D::turn_on_display(&mut self.interface)?;

            defmt::info!("show frame !");
        }
        Ok(())
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn clear_display(&mut self, color: BinaryColor) -> Result<(), D::Error> {
        D::restore_normal_mode(&mut self.interface)?;
        for fb in self.framebufs.iter_mut() {
            fb.fill(color);
        }
        D::update_frame(&mut self.interface, self.framebufs[0].as_bytes())?;
        D::turn_on_display(&mut self.interface)?;
        Ok(())
    }
}

impl<'a, C, DI: DisplayInterface, S: DisplaySize, D: GrayScaleDriver<C>> DrawTarget
    for GrayScaleEPD<'a, C, DI, S, D>
where
    [(); S::N]:,
    [(); C::BITS_PER_PIXEL]:,

    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
{
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            let mut l = color.luma();
            for i in 0..C::BITS_PER_PIXEL {
                let bit = l & 0x01;
                l >>= 1;
                self.framebufs[i].draw_iter([Pixel(
                    point,
                    if bit == 0 {
                        BinaryColor::On
                    } else {
                        BinaryColor::Off
                    },
                )])?;
            }
        }
        Ok(())
    }
}

impl<'a, C, DI: DisplayInterface, S: DisplaySize, D: GrayScaleDriver<C>> Dimensions
    for GrayScaleEPD<'a, C, DI, S, D>
where
    [(); S::N]:,
    [(); C::BITS_PER_PIXEL]:,

    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebufs[0].bounding_box()
    }
}
