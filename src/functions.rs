use crate::{cell::ImageCell, core::Image, merger::Point, BufferedImage};
use image::Pixel;
use rayon::{
    iter::IntoParallelIterator,
    prelude::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};
use std::{marker::Sync, ops::DerefMut};

/// The smallest amount of work, counted in subpixels, that is worth handing to another thread.
/// Splitting any finer than this spends more time scheduling tasks than it saves in throughput,
/// which is why both functions below work a row at a time rather than a pixel at a time.
const MIN_PARALLEL_SUBPIXELS: usize = 256 * 1024;

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
    // A row of the image being pasted is contiguous in memory, and so is the run of canvas it
    // lands on, so a whole row can be moved in one copy instead of a write per pixel.
    let row_length = top.width() as usize * <P as Pixel>::CHANNEL_COUNT as usize;
    if row_length == 0 {
        return;
    }

    let minimum_rows = (MIN_PARALLEL_SUBPIXELS / row_length).max(1);

    // The container behind an image is allowed to be larger than the image itself, so only the
    // rows that actually belong to it are copied.
    top.par_chunks_exact(row_length)
        .take(top.height() as usize)
        .with_min_len(minimum_rows)
        .enumerate()
        .for_each(|(row, subpixels)| unsafe {
            // SAFETY: Every row of the image being pasted lands on its own run of the canvas, so
            // no two threads ever write to the same memory location.
            let mut handout = bottom.request_handout(loc.x, loc.y + row as u32);
            handout.unsafe_put_row(subpixels);
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

    // Every row of the new image samples the same set of source columns, so the column each pixel
    // reads from is worth working out once up front rather than once per pixel.
    let source_columns: Vec<u32> = (0..nwidth)
        .map(|i| (i as f32 * width_ratio) as u32)
        .collect();

    let row_length = nwidth as usize * <P as Pixel>::CHANNEL_COUNT as usize;
    let minimum_rows = (MIN_PARALLEL_SUBPIXELS / row_length.max(1)).max(1);

    // Handing out a whole row at a time keeps each thread's writes sequential in the underlying
    // buffer, which the previous column-major walk did not.
    (0..nheight)
        .into_par_iter()
        .with_min_len(minimum_rows)
        .for_each(|j| {
            let y = (j as f32 * height_ratio) as u32;

            for (i, x) in source_columns.iter().enumerate() {
                let pixel = image.get_pixel(*x, y);

                unsafe {
                    // SAFETY: Each thread owns a whole row of the new image, so two threads will
                    // never write to the same memory location.

                    // Does this run around the borrow checker and violate the entire point of the Rust language?
                    // Yes, but we're here for raw speed :)
                    let mut handout = cell.request_handout(i as u32, j);
                    handout.unsafe_put_pixel(pixel)
                }
            }
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
