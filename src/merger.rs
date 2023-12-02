use image::Pixel;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::marker::Sync;
use sync_unsafe_cell::SyncUnsafeCell; // NOTE: Use this crate until std::cell::SyncUnsafeCell isn't nightly.

use crate::core::Image;

pub struct Merger<P: Pixel> {
    canvas: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted (images must be a uniform size)
    num_images: u32,              // The number of images that have been pasted to the canvas
    num_images_per_row: u32,      // The number of pages per row.
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    total_rows: u32,        // The total number of rows currently on the canvas.
}

impl<P: Pixel + Sync> Merger<P> {
    pub fn new(image_dimensions: (u32, u32), num_images_per_row: u32) -> Self {
        Self {
            canvas: Image::from(image::ImageBuffer::new(
                image_dimensions.0 * num_images_per_row,
                image_dimensions.1,
            )),
            image_dimensions: image_dimensions,
            num_images: 0,
            num_images_per_row: num_images_per_row,
            last_pasted_index: -1,
            total_rows: 1,
        }
    }

    pub fn get_num_images(&self) -> u32 {
        self.num_images
    }

    pub fn get_canvas(&self) -> &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> {
        &self.canvas
    }

    fn paste(
        &mut self,
        image: &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
        paste_x: u32,
        paste_y: u32,
    ) -> ()
    where
        <P as Pixel>::Subpixel: Sync + Send,
    {
        // Hold the contents of our canvas in a UnsafeCell so that each thread can mutate
        // its contents.
        let canvas_underlying = &*self.canvas.as_raw();
        let mut canvas_cell = SyncUnsafeCell::new(canvas_underlying);

        // Go through each pixel in the image (at once), grab its relatve location on the canvas,
        // and alter the canvas underlying buffer to reflect the new pixel.
        (0..image.width() * image.height())
            .into_par_iter()
            .for_each(|i| {
                let x = i % image.width();
                let y = i / image.width();

                let pixel = *image.get_pixel(x, y);

                // Get our canvas coordinates.
                let canvas_x = paste_x + x;
                let canvas_y = paste_y + y;

                // Obtain a mutable copy of the canvas cell
                // cannot borrow `canvas_cell` as mutable, as it is a captured variable in a `Fn` closure:: cannot borrow as mutable
                // TODO: Allow mutable variables in closures.
                // let canvas_cell_mut = canvas_cell.get_mut();
            })
    }

    fn get_next_paste_coordinates(&mut self) -> (u32, u32) {
        let available_images = (self.num_images_per_row * self.total_rows) - self.num_images;
        if available_images == 0 {
            panic!("No more space on canvas, please resize the canvas.");
        }

        // Calculate the next paste coordinates.
        let current_paste_index = (self.last_pasted_index + 1) as u32;
        let offset_x = current_paste_index % self.num_images_per_row;
        let offset_y = current_paste_index / self.num_images_per_row;

        let x = offset_x * self.image_dimensions.0;
        let y = offset_y * self.image_dimensions.1;

        return (x, y);
    }

    /// Allows the merger to push an image to the canvas. This can be used in a loop to paste a large number of images without
    /// having to hold all them in memory.
    pub fn push<U: image::GenericImage<Pixel = P>>(&mut self, image: &Image<P, U>) -> () {
        let (x, y) = self.get_next_paste_coordinates();

        image::imageops::overlay(&mut *self.canvas, &**image, x as i64, y as i64);

        self.last_pasted_index += 1;
        self.num_images += 1;
    }

    /// Allows the merger to bulk push N images to the canvas. This is useful for when you have a large number of images to paste.
    /// The downside is that you have to hold all of the images in memory at once, which can be a problem if you have a large number of images.
    pub fn bulk_push<U: image::GenericImage<Pixel = P> + std::marker::Sync>(
        &mut self,
        images: Vec<Image<P, U>>,
    ) {
        todo!();
    }

    /// Removes an image from the canvas at a given index. Indexing starts at 0 and works left to right, top to bottom.
    pub fn remove_image(&mut self, index: u32) {
        todo!()
    }
}
