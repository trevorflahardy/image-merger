use super::core::{MergerInfo, Padding};
use crate::{cell::ImageCell, core::Image, functions::paste};
use image::Pixel;

/// A fixed size merger that will resize the image being pasted to fit according to the image
/// dimensions passed to the struct. This is useful when you do not know the size of the images
/// being pasted but want then to all be uniform on a canvas.
pub struct FixedSizeMerger<P: Pixel> {
    canvas: ImageCell<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted
    images_per_row: u32,          // The number of images per row.
    total_rows: u32,              // The total number of rows
    num_images: u32,              // The number of images that have been pasted to the canvas
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    pading: Option<Padding>,
}

impl<P: Pixel> MergerInfo for FixedSizeMerger<P> {
    fn get_images_per_row(&self) -> u32 {
        self.images_per_row
    }

    fn get_total_rows(&self) -> u32 {
        self.total_rows
    }

    fn get_image_dimensions(&self) -> (u32, u32) {
        self.image_dimensions
    }
}
