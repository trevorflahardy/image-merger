//! This crate provides faster functionality for image processing. It is build on top of the image crate and aims to provide
//! a very fast way to merge many images together. It does this this by utilizing parallel processing and by avoiding
//! unnecessary copies.
//!
//! The main type of this crate is the [KnownSizeMerger](crate::KnownSizeMerger) struct, but, more will be added in the future.
mod cell;
mod core;
mod functions;
mod merger;

pub use crate::core::*;
pub use crate::merger::*;
pub use image::{Luma, LumaA, Pixel, Rgb, Rgba};

/// Unsafe functions and types that are used internally by this crate. These are exposed for advanced users who want to
/// implement their own merger. These functions and types are not guaranteed to be stable.
pub mod raw {
    pub use crate::cell::*;
    pub use crate::functions::*;
}
