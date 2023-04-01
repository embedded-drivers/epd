#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_arg_infer)]

pub mod display;
// pub mod drivers;
pub mod interface;

use core::marker::PhantomData;

use display::{DisplaySize, FrameBuffer};
use drivers::Driver;
use embedded_graphics::{
    prelude::{Dimensions, DrawTarget},
    primitives::Rectangle,
};
use interface::DisplayInterface;
pub use interface::EPDInterface;

pub mod drivers;

pub struct EPD<
    I: DisplayInterface,
    S: DisplaySize,
    D: Driver,
    const WIDTH: usize = { S::WIDTH },
    const HEIGHT: usize = { S::HEIGHT },
>
//[(); (WIDTH / 8 + (WIDTH % 8 != 0) as usize) * HEIGHT]:,
where
    [(); { (WIDTH / 8 + (WIDTH % 8 != 0) as usize) * HEIGHT }]:,
{
    pub interface: I,
    pub framebuf: FrameBuffer<WIDTH, HEIGHT>,
    _phantom: PhantomData<(S, D)>,
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver> EPD<I, S, D> {
    pub fn new(interface: I) -> Self {
        Self {
            interface,
            framebuf: FrameBuffer::new(),
            _phantom: PhantomData,
        }
    }

    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), D::Error>
    where
        DELAY: embedded_hal::blocking::delay::DelayUs<u32>,
    {
        D::wake_up(&mut self.interface, delay)
    }

    //pub fn update_frame(&mut self) -> Result<(), D::Error> {
    //  }

    pub fn display_frame(&mut self) -> Result<(), D::Error> {
        D::update_frame(&mut self.interface, 0, self.framebuf.as_bytes())?;
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
        D::wake_up(&mut self.interface, delay)
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver, const WIDTH: usize, const HEIGHT: usize>
    Dimensions for EPD<I, S, D, WIDTH, HEIGHT>
where
    [(); { (WIDTH / 8 + (WIDTH % 8 != 0) as usize) * HEIGHT }]:,
{
    fn bounding_box(&self) -> Rectangle {
        self.framebuf.bounding_box()
    }
}

impl<I: DisplayInterface, S: DisplaySize, D: Driver, const WIDTH: usize, const HEIGHT: usize>
    DrawTarget for EPD<I, S, D, WIDTH, HEIGHT>
where
    [(); { (WIDTH / 8 + (WIDTH % 8 != 0) as usize) * HEIGHT }]:,
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
