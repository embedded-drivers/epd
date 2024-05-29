#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_arg_infer)]

pub mod color;
pub mod display;
pub mod drivers;
pub mod interface;

use core::marker::PhantomData;

use color::GrayColorInBits;
pub use color::TriColor;
use defmt::println;
use display::{DisplaySize, FrameBuffer, GrayFrameBuffer};
use drivers::{Driver, FastUpdateDriver, GrayScaleDriver, MultiColorDriver};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, GrayColor, PixelColor},
    primitives::Rectangle,
    Pixel,
};
use interface::DisplayInterface;
pub use interface::EpdInterface;

use crate::drivers::WaveformDriver;

pub struct Epd<I: DisplayInterface, S: DisplaySize, D: Driver>
where
    [(); S::N]:,
{
    pub interface: I,
    pub framebuf: FrameBuffer<S>,
    _phantom: PhantomData<(S, D)>,
}

impl<DI: DisplayInterface, S: DisplaySize, D: Driver> Epd<DI, S, D>
where
    [(); S::N]:,
{
    pub fn new(interface: DI) -> Self {
        Self {
            interface,
            framebuf: if D::BLACK_BIT == false {
                FrameBuffer::new_ones()
            } else {
                FrameBuffer::new()
            },
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.framebuf.set_rotation(rotation);
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::update_frame(&mut self.interface, self.framebuf.as_bytes())?;
        D::turn_on_display(&mut self.interface)
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn wake_up<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> Dimensions for Epd<I, S, D>
where
    [(); S::N]:,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf.bounding_box()
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> DrawTarget for Epd<I, S, D>
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

/// EPD display backed by fast update LUT, both fast update and full update are supported.
pub struct FastUpdateEpd<I: DisplayInterface, S: DisplaySize, D: FastUpdateDriver>
where
    [(); S::N]:,
{
    pub interface: I,
    pub framebuf: FrameBuffer<S>,
    _phantom: PhantomData<(S, D)>,
}

impl<DI: DisplayInterface, S: DisplaySize, D: FastUpdateDriver> FastUpdateEpd<DI, S, D>
where
    [(); S::N]:,
{
    pub fn new(interface: DI) -> Self {
        Self {
            interface,
            framebuf: if D::BLACK_BIT == false {
                FrameBuffer::new_ones()
            } else {
                FrameBuffer::new()
            },
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        D::setup_fast_waveform(&mut self.interface)?;
        Ok(())
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.framebuf.set_rotation(rotation);
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::update_frame(&mut self.interface, self.framebuf.as_bytes())?;
        <D as WaveformDriver>::turn_on_display(&mut self.interface)?;
        Ok(())
    }

    pub fn display_frame_full_update(&mut self) -> Result<(), D::Error> {
        D::restore_normal_waveform(&mut self.interface)?;
        D::update_frame(&mut self.interface, self.framebuf.as_bytes())?;
        <D as WaveformDriver>::turn_on_display(&mut self.interface)?;
        D::setup_fast_waveform(&mut self.interface)?;
        Ok(())
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn wake_up<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: FastUpdateDriver> Dimensions for FastUpdateEpd<I, S, D>
where
    [(); S::N]:,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf.bounding_box()
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: FastUpdateDriver> DrawTarget for FastUpdateEpd<I, S, D>
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

pub struct TriColorEpd<I: DisplayInterface, S: DisplaySize, D: Driver>
where
    [(); S::N]:,
{
    pub interface: I,
    pub framebuf0: FrameBuffer<S>,
    pub framebuf1: FrameBuffer<S>,
    _phantom: PhantomData<(S, D)>,
}

impl<DI: DisplayInterface, S: DisplaySize, D: MultiColorDriver> TriColorEpd<DI, S, D>
where
    [(); S::N]:,
{
    pub fn new(interface: DI) -> Self {
        Self {
            interface,
            framebuf0: FrameBuffer::new_ones(),
            framebuf1: FrameBuffer::new(),
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;

        Ok(())
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.framebuf0.set_rotation(rotation);
        self.framebuf1.set_rotation(rotation);
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        defmt::info!("B/W {:?}", &self.framebuf0.as_bytes()[0..10]);
        defmt::info!("RED {:?}", &self.framebuf1.as_bytes()[0..10]);
        D::update_channel_frame(&mut self.interface, 0, self.framebuf0.as_bytes())?;
        D::update_channel_frame(&mut self.interface, 1, self.framebuf1.as_bytes())?;
        D::turn_on_display(&mut self.interface)
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn wake_up<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, S::WIDTH as _, S::HEIGHT as _)?;
        Ok(())
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> Dimensions for TriColorEpd<I, S, D>
where
    [(); S::N]:,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf0.bounding_box()
    }
}

impl<I: DisplayInterface, SIZE: DisplaySize, D: Driver> DrawTarget for TriColorEpd<I, SIZE, D>
where
    [(); SIZE::N]:,
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

pub struct GrayScaleEpd<C, I: DisplayInterface, SIZE: DisplaySize, D: GrayScaleDriver<C>>
where
    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
    [(); SIZE::N]:,
    [(); C::BITS_PER_PIXEL]:,
    [(); SIZE::N * C::BITS_PER_PIXEL]:,
{
    pub interface: I,
    pub framebuf: GrayFrameBuffer<SIZE, C>,
    _phantom: PhantomData<D>,
}

impl<'a, C, I: DisplayInterface, SIZE: DisplaySize, D: GrayScaleDriver<C>>
    GrayScaleEpd<C, I, SIZE, D>
where
    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
    [(); SIZE::N]:,
    [(); C::BITS_PER_PIXEL]:,
    [(); SIZE::N * C::BITS_PER_PIXEL]:,
{
    pub fn new(interface: I) -> Self {
        Self {
            interface,
            framebuf: GrayFrameBuffer::new(),
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::wake_up(&mut self.interface, delay)?;
        D::set_shape(&mut self.interface, SIZE::WIDTH as _, SIZE::HEIGHT as _)?;

        Ok(())
    }

    pub fn set_rotation(&mut self, rotation: i32) {
        self.framebuf.set_rotation(rotation);
    }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::setup_gray_scale_waveform(&mut self.interface)?;

        let width_in_byte = SIZE::WIDTH / 8 + (SIZE::WIDTH % 8 != 0) as usize;

        for i in (0..C::MAX_VALUE + 1).rev() {
            defmt::debug!("display layer {}", i);
            let mut tmp = [0xffu8; SIZE::N];
            // extract gray channel and fill in the tmp buffer
            for y in 0..SIZE::HEIGHT {
                for x in 0..SIZE::WIDTH {
                    let byte_offset = y * width_in_byte + x / 8;
                    let bit_offset = 7 - x % 8;

                    let pixel = self.framebuf.get_pixel_in_raw_pos(x, y);

                    let val = pixel.luma(); // 0, 1, 2, 3
                                            // defmt::info!("x {} y {}  val {}", x, y, val);

                    if val == 7 {
                        // defmt::info!("layer 7");
                    }
                    if val < i {
                        tmp[byte_offset] &= !(1 << bit_offset);
                        //tmp[byte_offset] |= (1 << bit_offset);
                    }
                }
            }
            println!("frame {}", tmp.iter().filter(|&&x| x != 0xff).count());
            D::update_frame(&mut self.interface, &tmp)?;
            <D as WaveformDriver>::turn_on_display(&mut self.interface)?;
        }

        Ok(())
    }

    pub fn sleep<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::delay::DelayNs,
    {
        D::sleep(&mut self.interface, delay)
    }

    pub fn clear_display(&mut self, color: BinaryColor) -> Result<(), D::Error> {
        D::restore_normal_waveform(&mut self.interface)?;

        self.framebuf.fill(color);

        D::update_frame(&mut self.interface, self.framebuf.as_bytes())?;
        <D as Driver>::turn_on_display(&mut self.interface)?;
        Ok(())
    }
}

impl<C, DI: DisplayInterface, S: DisplaySize, D: GrayScaleDriver<C>> DrawTarget
    for GrayScaleEpd<C, DI, S, D>
where
    [(); S::N]:,
    [(); C::BITS_PER_PIXEL]:,
    [(); S::N * C::BITS_PER_PIXEL]:,

    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
{
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            self.framebuf.set_pixel(point.x as _, point.y as _, color);
        }
        Ok(())
    }
}

impl<C, DI: DisplayInterface, S: DisplaySize, D: GrayScaleDriver<C>> Dimensions
    for GrayScaleEpd<C, DI, S, D>
where
    [(); S::N]:,
    [(); C::BITS_PER_PIXEL]:,
    [(); S::N * C::BITS_PER_PIXEL]:,

    C: GrayColor + GrayColorInBits + PixelColor + From<<C as PixelColor>::Raw>,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf.bounding_box()
    }
}
