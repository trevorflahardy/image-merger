use crate::{cell::ImageCell, core::Image, merger::Point};
use image::Pixel;
use rayon::{
    prelude::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use std::{marker::Sync, ops::DerefMut};

/// The library's underlying paste method.
/// # Arguments
/// * `bottom` - The image to paste onto.
/// * `top` - The image to paste.
/// * `loc` - The location to paste the top image at.
pub fn paste<P, Container>(
    bottom: &ImageCell<P, image::ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>>,
    top: &Image<P, image::ImageBuffer<P, Container>>,
    loc: Point,
) where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    Container: DerefMut<Target = [P::Subpixel]>,
{
    // Go through each pixel in the image (at once), grab its relatve location on the canvas,
    // and alter the canvas underlying buffer to reflect the new pixel.
    let image_width = top.width();
    top.par_chunks_exact(<P as Pixel>::CHANNEL_COUNT as usize)
        .enumerate()
        .for_each(|(index, chunk)| {
            let x = index as u32 % image_width;
            let y = index as u32 / image_width;

            let canvas_x = loc.x + x;
            let canvas_y = loc.y + y;

            let pixel = <P as Pixel>::from_slice(chunk);
            unsafe {
                let mut handout = bottom.request_handout(canvas_x, canvas_y);
                handout.unsafe_put_pixel(*pixel);
            }
        });
}

/// The library's underlying rezsize method for nearest neighbor. This function
/// runs in O(n), but it is done in parallel to speed up the process in larger images.
/// The drawback to this is that the image is not resized in place, but rather a new
/// one is created - so you will hold two images in memory at once.
/// # Arguments
/// * `image` - The image to resize.
/// * `new_width` - The new width of the image.
/// * `new_height` - The new height of the image.
pub fn resize_nearest_neighbor<P, Container>(
    _image: &Image<P, image::ImageBuffer<P, Container>>,
    _new_width: u32,
    _new_height: u32,
) -> Image<P, image::ImageBuffer<P, Container>>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    Container: DerefMut<Target = [P::Subpixel]>,
{
    todo!();
}
