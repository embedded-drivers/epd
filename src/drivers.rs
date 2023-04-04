use core::iter;

use crate::interface::{DisplayError, DisplayInterface};
use embedded_graphics::prelude::GrayColor;
use embedded_hal::blocking::delay::DelayUs;

pub use self::ssd1608::*;
pub use self::ssd1619a::*;
pub use self::ssd1680::*;

mod ssd1608;
mod ssd1619a;
mod ssd1680;

pub trait Driver {
    type Error;

    // Almost all EPD use bit 0 as black, but some use bit 1 as black
    const BLACK_BIT: bool = false;

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

pub trait FastUpdateDriver: WaveformDriver {}

pub trait GrayScaleDriver<Color: GrayColor>: WaveformDriver {
    fn init_as_gray_scale<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        Ok(())
    }
    // const LUT_FULL_UPDATE: &'static [u8];
    // const LUT_FRAME_UPDATE: &'static [u8];
    fn setup_gray_scale_waveform<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;

    fn restore_normal_mode<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error>;
}

/// IL0373?
/// Up to 160 source x 296 gate resolution
/// small, including 420 and 437
/// Pervasive Displays
// https://github.com/rei-vilo/PDLS_EXT3_Basic/blob/main/src/Screen_EPD_EXT3.cpp
pub struct PLDS;

impl Driver for PLDS {
    type Error = DisplayError;

    fn busy_wait<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        // negative logic
        while !di.is_busy_on() {}
        Ok(())
    }

    fn wake_up<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.reset(delay, 10_000, 10_000);
        Self::busy_wait(di)?;

        di.send_command_data(0x00, &[0x0e])?; // soft-reset

        delay.delay_us(5_000_u32);
        di.send_command_data(0xe5, &[0x16]).unwrap(); // Input Temperature 0°C = 0x00, 22°C = 0x16, 25°C = 0x19

        di.send_command_data(0xe0, &[0x02]).unwrap(); // Active Temperature

        Ok(())
    }

    fn set_shape<DI: DisplayInterface>(di: &mut DI, x: u16, y: u16) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_frame<'a, DI: DisplayInterface, I>(di: &mut DI, buffer: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        di.send_command(0x10)?;
        let n = di.send_data_from_iter(buffer)?;

        // empty red channel
        di.send_command(0x13)?;
        di.send_data_from_iter(iter::repeat(&0).take(n))?;
        Ok(())
    }

    fn turn_on_display<DI: DisplayInterface>(di: &mut DI) -> Result<(), Self::Error> {
        di.send_command_data(0x04, &[0x00])?; // Power on

        Self::busy_wait(di)?;

        di.send_command_data(0x12, &[0x00])?; // display refresh
        Self::busy_wait(di)?;

        Ok(())
    }

    fn sleep<DI: DisplayInterface, DELAY: DelayUs<u32>>(
        di: &mut DI,
        delay: &mut DELAY,
    ) -> Result<(), Self::Error> {
        di.send_command_data(0x02, &[0x00])?; // turn off dc/dc
        delay.delay_us(5_000_u32);
        Self::busy_wait(di)?;

        Ok(())
    }
}

impl MultiColorDriver for PLDS {
    fn update_channel_frame<'a, DI: DisplayInterface, I>(
        di: &mut DI,
        channel: u8,
        buffer: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = &'a u8>,
    {
        if channel == 0 {
            di.send_command(0x10)?;
            di.send_data_from_iter(buffer)?;
        } else if channel == 1 {
            di.send_command(0x13)?;
            di.send_data_from_iter(buffer)?;
        } else {
            //
        }
        Ok(())
    }
}
