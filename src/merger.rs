use image::Pixel;

use crate::image::{Image, RgbaImageBuffer};

/// NOTE FOR FUTURE: Canvas dimensions will dynamically resize based on if you hit an index that is out of bounds. This means a new canvas will only
/// contain space for num_images_per_row and resize later. To combat this, the canvas can be resized according to the expected number of images. This is a memory
/// management feature

pub struct Merger {
    canvas: RgbaImageBuffer<Vec<u8>>, // The canvas that gets written to.
    canvas_dimensions: (u32, u32), // The end dimensions of the canvas (can be resized if dynamically pasting)
    image_dimensions: (u32, u32), // The dimensions of the images being pasted (images must be a uniform size)
    num_images: u32,              // The number of images that have been pasted to the canvas
    num_images_per_row: u32,      // The number of pages per row.
    past_pasted_index: f32, // The index of the last pasted image, starts at -1 if not images have been pasted.
}

impl Merger {
    pub fn pasted_images_len(&self) -> u32 {
        self.num_images
    }

    fn paste<T: image::GenericImage, F: image::GenericImageView>(
        &self,
        to: &mut T,
        from: &F,
        x: u32,
        y: u32,
    ) {
        // NOTE: Maybe instead of directly pasting "to" then we could collect all pixels together in a vector and build a new image from that,
        // could be faster because of threading?
        todo!()
    }

    /// Allows the merger to push an image to the canvas. This can be used in a loop to paste a large number of images without
    /// having to hold all them in memory.
    pub fn push<P: Pixel, U: image::GenericImage<Pixel = P>>(&mut self, image: &Image<P, U>) {
        todo!()
    }

    /// Allows the merger to bulk push N images to the canvas. This is useful for when you have a large number of images to paste.
    /// The downside is that you have to hold all of the images in memory at once, which can be a problem if you have a large number of images.
    pub fn bulk_push<P: Pixel, U: image::GenericImage<Pixel = P>>(
        &mut self,
        images: Vec<Image<P, U>>,
    ) {
        todo!()
    }

    /// Removes an image from the canvas at a given index. Indexing starts at 0 and works left to right, top to bottom.
    pub fn remove_image(&mut self, index: u32) {
        todo!()
    }
}
