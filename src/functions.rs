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

    /// Builds an image in which no two pixels are alike, so that a resize which samples the wrong
    /// place produces a different result. A solid colour cannot tell one sampling position from
    /// another, and so cannot test a resize at all.
    fn varied_image(width: u32, height: u32) -> BufferedImage<Rgba<u8>> {
        let mut image: BufferedImage<Rgba<u8>> = Image::new(width, height);

        for x in 0..width {
            for y in 0..height {
                image.put_pixel(
                    x,
                    y,
                    Rgba([x as u8, y as u8, (x * 7 + y * 13) as u8, (x + y) as u8]),
                );
            }
        }

        image
    }

    /// Checks a resize against the sampling this crate performs, which takes the source pixel at
    /// `floor(index * ratio)`.
    ///
    /// Note that this is not where the image crate's Nearest filter samples, which takes the pixel
    /// nearest the centre of the destination pixel instead. The two agree only on a solid colour,
    /// so the expected pixels are worked out from the ratios rather than by comparing against it.
    fn assert_resizes_to(source: &BufferedImage<Rgba<u8>>, nwidth: u32, nheight: u32) {
        let resized = resize_nearest_neighbor(source, nwidth, nheight);

        assert_eq!(resized.width(), nwidth);
        assert_eq!(resized.height(), nheight);

        let width_ratio = source.width() as f32 / nwidth as f32;
        let height_ratio = source.height() as f32 / nheight as f32;

        for i in 0..nwidth {
            for j in 0..nheight {
                let x = (i as f32 * width_ratio) as u32;
                let y = (j as f32 * height_ratio) as u32;

                assert_eq!(
                    resized.get_pixel(i, j),
                    source.get_pixel(x, y),
                    "pixel ({i}, {j}) of a {nwidth}x{nheight} resize sampled the wrong place"
                );
            }
        }
    }

    #[test]
    fn test_resize_nearest_neighbor() {
        assert_resizes_to(&varied_image(100, 100), 50, 50);
    }

    #[test]
    fn test_resize_nearest_neighbor_non_square() {
        // A square resize hides a width and height that have been swapped, so both orientations
        // are checked here.
        assert_resizes_to(&varied_image(97, 43), 31, 67);
        assert_resizes_to(&varied_image(43, 97), 67, 31);
    }

    #[test]
    fn test_resize_nearest_neighbor_upscale_and_degenerate() {
        assert_resizes_to(&varied_image(64, 48), 128, 96);
        assert_resizes_to(&varied_image(7, 7), 1, 1);
        assert_resizes_to(&varied_image(1, 1), 9, 9);
        assert_resizes_to(&varied_image(256, 256), 255, 257);
    }

    #[test]
    fn test_resize_nearest_neighbor_above_the_parallel_threshold() {
        // Large enough that the resize is handed to more than one thread, which none of the
        // smaller cases above are.
        assert_resizes_to(&varied_image(1024, 1024), 512, 512);
    }

    #[test]
    fn test_resize_nearest_neighbor_samples_the_start_of_each_block() {
        // Written out by hand so that the expected pixels do not come from the same arithmetic the
        // implementation uses. Halving a 4x4 image samples columns and rows 0 and 2.
        let mut image: BufferedImage<Rgba<u8>> = Image::new(4, 4);
        for x in 0..4 {
            for y in 0..4 {
                image.put_pixel(x, y, Rgba([((x * 10) + y) as u8, 0, 0, 255]));
            }
        }

        let resized = resize_nearest_neighbor(&image, 2, 2);

        assert_eq!(
            resized.get_pixel(0, 0).0[0],
            0,
            "should sample source (0, 0)"
        );
        assert_eq!(
            resized.get_pixel(1, 0).0[0],
            20,
            "should sample source (2, 0)"
        );
        assert_eq!(
            resized.get_pixel(0, 1).0[0],
            2,
            "should sample source (0, 2)"
        );
        assert_eq!(
            resized.get_pixel(1, 1).0[0],
            22,
            "should sample source (2, 2)"
        );
    }
}
