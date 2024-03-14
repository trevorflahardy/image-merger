use crate::core::Image;
use image::Pixel;
use std::{marker::Sync, ops::DerefMut};

/// Represents a point on any canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

/// Represents the padding between images on a canvas.
/// # Fields
/// * `x` - The padding between images on the x axis.
/// * `y` - The padding between images on the y axis.
pub type Padding = Point;

/// The Merger trait that all mergers must implement. This trait allows the merger to paste images to a canvas.
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
pub trait Merger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    /// Returns a reference to the underlying canvas.
    fn get_canvas(&self) -> &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>;

    /// Consumes the underlying merger and returns the canvas.
    fn into_canvas(self) -> Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>;

    /// Allows the merger to push an image to the canvas. This can be used in a loop to paste a large number of images without
    /// having to hold all them in memory.
    /// # Arguments
    /// * `image` - The image to push onto the canvas. Its pixel type, `P`, must match the canvas, and its `Container` must be dereferenceable to
    /// a slice of `P::Subpixel`s.
    fn push<Container>(&mut self, image: &Image<P, image::ImageBuffer<P, Container>>)
    where
        Container: DerefMut<Target = [P::Subpixel]>;

    /// Allows the merger to bulk push N images to the canvas. This is useful for when you have a large number of images to paste.
    /// The downside is that you have to hold all of the images in memory at once, which can be a problem if you have a large number of images.
    /// # Arguments
    /// * `images` - The images to push onto the canvas. Note that the argument type is `&[&Image<...>]`, the func
    /// does not need to take ownership of the images, it only needs to read them. The pixel type, `P`, of the images must match the canvas, and
    /// their `Container` must be dereferenceable to a slice of `P::Subpixel`s.
    fn bulk_push<Container>(&mut self, images: &[&Image<P, image::ImageBuffer<P, Container>>])
    where
        Container: DerefMut<Target = [P::Subpixel]> + Sync;
}
