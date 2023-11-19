use image::Pixel;
use num_traits::Zero;

use crate::core::Image;

/// NOTE FOR FUTURE: Canvas dimensions will dynamically resize based on if you hit an index that is out of bounds. This means a new canvas will only
/// contain space for num_images_per_row and resize later. To combat this, the canvas can be resized according to the expected number of images. This is a memory
/// management feature

pub struct Merger<P: Pixel> {
    canvas: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted (images must be a uniform size)
    num_images: u32,              // The number of images that have been pasted to the canvas
    num_images_per_row: u32,      // The number of pages per row.
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    total_rows: u32,        // The total number of rows currently on the canvas.
}

impl<P: Pixel> Merger<P> {
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

    fn grow_canvas(&mut self) -> () {
        self.total_rows += 1;

        let new_canvas_dimensions = (self.canvas.width(), self.canvas.height() * self.total_rows);

        // Create a new container with the capacity of the new canvas
        let updated_capacity = (<P as Pixel>::CHANNEL_COUNT as usize)
            * (new_canvas_dimensions.0 * new_canvas_dimensions.1) as usize;

        // Steal the data from the immutable reference to the underlying container into a new vector
        // that we can transfer to a new container.
        let mut new_container: Vec<P::Subpixel> = Vec::with_capacity(updated_capacity);
        unsafe {
            new_container.set_len(updated_capacity);
            let stolen_container_ptr = (*self.canvas.get_underlying_mut()).as_mut_ptr();
            let new_container_ptr = new_container.as_mut_ptr();
            std::ptr::copy_nonoverlapping(
                stolen_container_ptr,
                new_container_ptr,
                self.canvas.capacity(),
            );
        }

        // Fill the new container with zeros.
        new_container.resize(updated_capacity, P::Subpixel::zero());

        // Check if the image will fit

        // Create a new canvas with the new dimensions and the new container.
        let new_canvas: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> = Image::from(
            image::ImageBuffer::from_raw(
                new_canvas_dimensions.0,
                new_canvas_dimensions.1,
                new_container,
            )
            .unwrap(),
        );
        self.canvas = new_canvas;
    }

    fn get_next_paste_coordinates(&mut self) -> (u32, u32) {
        let available_images = (self.num_images_per_row * self.total_rows) - self.num_images;
        if available_images == 0 {
            // Resize the canvas to make room for the next row, we are out of space.
            self.grow_canvas();
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

        let canvas = self.canvas.get_underlying_mut();
        image::imageops::overlay(&mut *canvas, image.get_underlying(), x as i64, y as i64);

        self.last_pasted_index += 1;
        self.num_images += 1;
    }

    /// Allows the merger to bulk push N images to the canvas. This is useful for when you have a large number of images to paste.
    /// The downside is that you have to hold all of the images in memory at once, which can be a problem if you have a large number of images.
    pub fn bulk_push<U: image::GenericImage<Pixel = P>>(&mut self, images: Vec<Image<P, U>>) {
        todo!()
    }

    /// Removes an image from the canvas at a given index. Indexing starts at 0 and works left to right, top to bottom.
    pub fn remove_image(&mut self, index: u32) {
        todo!()
    }
}
