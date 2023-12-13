use crate::{cell::ImageCell, core::Image, merger::Point};
use image::Pixel;
use rayon::{
    prelude::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use std::{marker::Sync, ops::DerefMut};

/// The library's underlying paste method. It requires you to pass the bit
pub fn paste<P, Container>(
    bottom: &ImageCell<P, image::ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>>,
    top: &Image<P, image::ImageBuffer<P, Container>>,
    loc: Point,
) -> ()
where
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
                let mut handout = bottom.request_handout_unchecked(canvas_x, canvas_y);
                handout.put_pixel(*pixel);
            }
        });
}