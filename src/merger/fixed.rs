use super::core::Padding;
use crate::{cell::ImageCell, core::Image, functions::paste};
use image::Pixel;

/// A fixed size merger that will resize the image being pasted to fit according to the image
/// dimensions passed to the struct. This is useful when you do not know the size of the images
/// being pasted but want then to all be uniform on a canvas.
pub struct FixedSizeMerger<P: Pixel> {
    canvas: ImageCell<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted
    images_per_row: u32,          // The number of images per row.
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    total_rows: u32,        // The total number of rows currently on the canvas.
    pading: Option<Padding>,
}
