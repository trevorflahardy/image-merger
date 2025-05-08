use crate::{cell::ImageCell, core::Image, merger::Point, BufferedImage};
use image::Pixel;
use rayon::{
    iter::IntoParallelIterator,
    prelude::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use std::{marker::Sync, ops::DerefMut};

/// The library's underlying paste method. This is only used internally and should not be used by the user, but is exposed
/// through the raw module for documentation purposes.
/// # Arguments
/// * `bottom` - The image to paste onto.
/// * `top` - The image to paste.
/// * `loc` - The location to paste the top image at.
pub fn paste<P, Container>(
    bottom: &ImageCell<P, image::ImageBuffer<P, Container>>,
    top: &Image<P, image::ImageBuffer<P, Container>>,
    loc: Point,
) where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    Container: DerefMut<Target = [P::Subpixel]>,
{
    // Go through each pixel in the image (at once), grab its relative location on the canvas,
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

/// The library's underlying resize method. This is only used internally and should not be used by the user, but is exposed
/// through the raw module for documentation purposes.
/// # Arguments
/// * `image` - The image to resize.
/// * `nwidth` - The new width of the image.
/// * `nheight` - The new height of the image.
/// # Returns
/// * A new image with the new dimensions. Note that the returned image's underlying buffer is not guaranteed to be the same as the input image's buffer. The returned buffer will be `Vec` based.
pub fn resize_nearest_neighbor<P, U>(
    image: &Image<P, U>,
    nwidth: u32,
    nheight: u32,
) -> BufferedImage<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    U: image::GenericImage<Pixel = P> + Sync,
{
    // Create a cell to hold the new image.
    let new_image: Image<P, _> = Image::new(nwidth, nheight);
    let cell = ImageCell::new(new_image);

    // Grab the ratios of the new image to the old image.
    let height_ratio = image.height() as f32 / nheight as f32;
    let width_ratio = image.width() as f32 / nwidth as f32;

    (0..nwidth).into_par_iter().for_each(|i| {
        (0..nheight).into_par_iter().for_each(|j| {
            let x = (i as f32 * width_ratio) as u32;
            let y = (j as f32 * height_ratio) as u32;

            let pixel = image.get_pixel(x, y);

            unsafe {
                // SAFETY: Each thread is writing to a different pixel in the image.
                // Two locations in memory will never be the same, so even through there may be (almost) concurrent
                // access to the same memory location, the offsets of changes will be different.

                // Does this run around the borrow checker and violate the entire point of the Rust language?
                // Yes, but we're here for raw speed :)
                let mut handout = cell.request_handout(i, j);
                handout.unsafe_put_pixel(pixel)
            }
        })
    });

    cell.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_resize_nearest_neighbor() {
        let mut image: Image<Rgba<u8>, _> = Image::new(100, 100);
        for i in 0..100 {
            for j in 0..100 {
                image.put_pixel(i, j, Rgba([255, 0, 0, 0]));
            }
        }

        let fast_resized = resize_nearest_neighbor(&image, 50, 50);
        let fast_resized_underlying = fast_resized.into_buffer();

        let image_underlying = image.clone();
        let slow_resized = image::imageops::resize(
            &image_underlying,
            50,
            50,
            image::imageops::FilterType::Nearest,
        );

        assert_eq!(fast_resized_underlying, slow_resized);
    }
}
