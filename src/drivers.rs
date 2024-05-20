use crate::interface::DisplayInterface;
use embedded_graphics::prelude::GrayColor;
use embedded_hal::delay::DelayNs;

pub use self::il3895::*;
pub use self::pd::*;
pub use self::ssd1608::*;
pub use self::ssd1619a::*;
pub use self::ssd1675b::*;
pub use self::ssd1680::*;
pub use self::uc8176::*;
pub use self::uc8179::*;

mod il3895;
mod pd;
mod ssd1608;
mod ssd1619a;
mod ssd1675b;
mod ssd1680;
mod uc8176;
mod uc8179;

pub type IL3820 = SSD1608;

pub trait Driver {
    type Error;

    // Almost all EPD use bit 0 as black, but some use bit 1 as black
    const BLACK_BIT: bool = false;

    /// Wake UP and init
    fn wake_up<DI: DisplayInterface, DELAY: DelayNs>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error>;

    // also set ram pos
    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error>;

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>;

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;

    fn sleep<DI: DisplayInterface, DELAY: DelayNs>(
        _di: &mut DI,
        _delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    // allow driver to override default busy wait
    fn busy_wait<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        while di.is_busy_on() {}
        Ok(())
    }
}

pub trait MultiColorDriver: Driver {
    fn init_multi_color<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_channel_frame<'a, DI: DisplayInterface, I>(
        di: &mut DI,
        channel: u8,
        buffer: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>;
}

pub trait WaveformDriver: Driver {
    // Some Drivers require a different Display Update Sequence for LUT loading
    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        <Self as Driver>::turn_on_display(di)
    }
    fn update_waveform<DI: DisplayInterface>(
        di: &mut DI,
        lut: &'static [u8],
    ) -> Result<(), Self::Error>;
}

pub trait FastUpdateDriver: WaveformDriver {
    fn setup_fast_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;
    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;
}

pub trait GrayScaleDriver<Color: GrayColor>: WaveformDriver {
    fn init_as_gray_scale<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        Ok(())
    }
    // const LUT_FULL_UPDATE: &'static [u8];
    // const LUT_FRAME_UPDATE: &'static [u8];
    fn setup_gray_scale_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;

    fn restore_normal_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;
}
