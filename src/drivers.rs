use crate::interface::{self, DisplayInterface};
use embedded_graphics::prelude::GrayColor;
use embedded_hal::blocking::delay::DelayUs;

pub use self::ssd1608::SSD1608;
pub use self::ssd1619a::SSD1619A;

mod ssd1608;
mod ssd1619a;

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
    // const LUT_FULL_UPDATE: &'static [u8];
    // const LUT_FRAME_UPDATE: &'static [u8];
    fn setup_gray_scale<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;

    fn restore_normal_mode<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;
}

