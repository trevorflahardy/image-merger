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

/// A convenience type alias for an [Image](Image) with a given [pixel][Pixel] type. This simplifies the type signature
/// of most Image declarations. This type alias assumes you are using a Vec<T> as the underlying image buffer.
///
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
/// # Example
/// ```
/// use image_merger::{Image, Rgba, BufferedImage};
/// use image::ImageBuffer;
///
/// // Declaring an image without a type alias.
/// let image: Image<Rgba<u8>, ImageBuffer<Rgba<u8>, Vec<u8>>> = Image::new(100, 100);
///
/// // Declaring an image with a type alias.
/// let image: BufferedImage<Rgba<u8>> = BufferedImage::new(100, 100);
pub type BufferedImage<P> = Image<P, ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>>;
