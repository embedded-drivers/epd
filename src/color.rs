pub use embedded_graphics::pixelcolor::{Gray2, Gray4, Gray8};
use embedded_graphics::prelude::{GrayColor, PixelColor};

/// 3 color display
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

// BITS_PER_PIXEL is hidden behind RawData. RawData for Gray3 is not possible now.
pub trait GrayColorInBits {
    const BITS_PER_PIXEL: usize;
    const MAX_VALUE: u8 = (1 << Self::BITS_PER_PIXEL) - 1;

    fn from_u8(value: u8) -> Self;
}

impl GrayColorInBits for Gray2 {
    const BITS_PER_PIXEL: usize = 2;

    fn from_u8(value: u8) -> Self {
        Gray2::new(value)
    }
}
impl GrayColorInBits for Gray4 {
    const BITS_PER_PIXEL: usize = 4;

    fn from_u8(value: u8) -> Self {
        Gray4::new(value)
    }
}

impl GrayColorInBits for Gray8 {
    const BITS_PER_PIXEL: usize = 8;

    fn from_u8(value: u8) -> Self {
        Gray8::new(value)
    }
}

/// 3 bit grayscale color
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gray3(u8);

impl Gray3 {
    pub fn new(value: u8) -> Self {
        Self::from_u8(value)
    }
}

impl From<u8> for Gray3 {
    fn from(value: u8) -> Self {
        Self::from_u8(value)
    }
}
// fake impl, to let compile pass
impl From<()> for Gray3 {
    fn from(_: ()) -> Self {
        Gray3(0)
    }
}

impl GrayColorInBits for Gray3 {
    const BITS_PER_PIXEL: usize = 3;

    fn from_u8(value: u8) -> Self {
        if value > 7 {
            return Gray3(7);
        }
        Gray3(value)
    }
}

// NOTE: RawData is a sealed trait in embedded-graphics, so we can't implement it for Gray3.
// Do not use Gray3 as storage for images. As it is not aligned to 8 bit boundaries.
impl PixelColor for Gray3 {
    type Raw = ();
}

impl GrayColor for Gray3 {
    fn luma(&self) -> u8 {
        self.0
    }

    const BLACK: Self = Gray3(0);

    const WHITE: Self = Gray3(0b111);
}
