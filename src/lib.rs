#![no_std]
#![feature(fixed_size_array, slice_fill)]

pub mod display;
pub mod drivers;
pub mod interface;

pub use interface::Interface;
