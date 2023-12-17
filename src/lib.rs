//! This crate provides fast functionality for merging many images.
//! It is built on top of the image crate and works to boost performance by utilizing parallel processing and
//! avoiding unnecessary costly operations.
//!
//! The main type of this crate is the [KnownSizeMerger](crate::KnownSizeMerger) struct, but, more will be added in the future.
mod cell;
mod core;
mod functions;
mod merger;

pub use crate::core::*;
pub use crate::merger::*;
pub use image::{ImageBuffer, Luma, LumaA, Pixel, Rgb, Rgba};

/// Unsafe functions and types that are used internally by this crate. These are exposed for advanced users who want to
/// implement their own merger. These functions and types are not guaranteed to be stable.
pub mod raw {
    pub use crate::cell::*;
    pub use crate::functions::*;
}
