use image::Pixel;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;
use std::marker::Sync;

use crate::{cell::ImageCell, core::Image};

pub struct Padding {
    pub x: u32,
    pub y: u32,
}

pub struct Merger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    canvas: ImageCell<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted (images must be a uniform size)
    num_images: u32,              // The number of images that have been pasted to the canvas
    images_per_row: u32,          // The number of pages per row.
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    total_rows: u32,        // The total number of rows currently on the canvas.
    padding: Option<Padding>,
}

#[allow(dead_code)]
impl<P> Merger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    pub fn new(
        image_dimensions: (u32, u32),
        images_per_row: u32,
        rows: u32,
        padding: Option<Padding>,
    ) -> Self {
        let image_gaps_x =
            (images_per_row - 1) * padding.as_ref().and_then(|p| Some(p.x)).unwrap_or(0);
        let image_gaps_y = (rows - 1) * padding.as_ref().and_then(|p| Some(p.y)).unwrap_or(0);

        let canvas: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> = Image::new(
            (image_dimensions.0 * images_per_row) + image_gaps_x,
            (image_dimensions.1 * rows) + image_gaps_y,
        );

        Self {
            canvas: ImageCell::new(canvas),
            image_dimensions: image_dimensions,
            num_images: 0,
            images_per_row: images_per_row,
            last_pasted_index: -1,
            total_rows: rows,
            padding,
        }
    }

    pub fn get_num_images(&self) -> u32 {
        self.num_images
    }

    pub fn get_canvas(&self) -> &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> {
        &self.canvas
    }

    fn paste(
        &self,
        image: &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
        paste_x: u32,
        paste_y: u32,
    ) -> () {
        // Hold the contents of our canvas in a UnsafeCell so that each thread can mutate
        // its contents.
        //let canvas_underlying = &*self.canvas.as_raw();
        let canvas_cell = &self.canvas;

        // Go through each pixel in the image (at once), grab its relatve location on the canvas,
        // and alter the canvas underlying buffer to reflect the new pixel.
        let image_width = image.width();
        let image_pixels = image.pixels().collect::<Vec<_>>();
        image_pixels
            .into_par_iter()
            .enumerate()
            .for_each(|(index, pixel)| {
                let x = index as u32 % image_width;
                let y = index as u32 / image_width;

                let canvas_x = paste_x + x;
                let canvas_y = paste_y + y;

                unsafe {
                    let mut handout = canvas_cell.request_handout_unchecked(canvas_x, canvas_y);
                    handout.put_pixel(pixel.clone());
                }
            });
    }

    #[inline(always)]
    fn additional_space(&self) -> u32 {
        (self.images_per_row * self.total_rows) - self.num_images
    }

    fn get_paste_coordinates_unchecked(&self, index: u32) -> (u32, u32) {
        let offset_x = index % self.images_per_row;
        let offset_y = index / self.images_per_row;

        let padding_x = self.padding.as_ref().and_then(|p| Some(p.x)).unwrap_or(0) * offset_x;
        let padding_y = self.padding.as_ref().and_then(|p| Some(p.y)).unwrap_or(0) * offset_y;

        let x = (offset_x * self.image_dimensions.0) + padding_x;
        let y = (offset_y * self.image_dimensions.1) + padding_y;

        (x, y)
    }

    fn get_next_paste_coordinates(&mut self) -> (u32, u32) {
        if self.additional_space() <= 0 {
            panic!("No more space on the canvas!");
        }

        self.get_paste_coordinates_unchecked((self.last_pasted_index + 1) as u32)
    }

    /// Allows the merger to push an image to the canvas. This can be used in a loop to paste a large number of images without
    /// having to hold all them in memory.
    pub fn push(&mut self, image: &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>) -> () {
        let (x, y) = self.get_next_paste_coordinates();

        self.paste(image, x, y);

        self.last_pasted_index += 1;
        self.num_images += 1;
    }

    /// Allows the merger to bulk push N images to the canvas. This is useful for when you have a large number of images to paste.
    /// The downside is that you have to hold all of the images in memory at once, which can be a problem if you have a large number of images.
    pub fn bulk_push(
        &mut self,
        images: &Vec<&Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>>,
    ) -> () {
        // If we can't fit all the images we need to panic.
        if self.additional_space() < images.len() as u32 {
            // TODO: Maybe only take as many images as we can fit?
            panic!("There is not enough space on the canvas to fit all the requested images.");
        }

        (0..images.len()).into_par_iter().for_each(|index| {
            let image = images[index];

            // The image coordinates can easily be calculated by using the last_pasted_index
            // and making the calculations ourselves.
            let offset_index = (index as i32 + self.last_pasted_index + 1) as u32;

            let (x, y) = self.get_paste_coordinates_unchecked(offset_index);
            self.paste(image, x, y);
        });

        self.last_pasted_index += images.len() as i32;
    }

    /// Removes an image from the canvas at a given index. Indexing starts at 0 and works left to right, top to bottom.
    pub fn remove_image(&mut self, index: u32) {
        let offset_x = index % self.images_per_row;
        let offset_y = index / self.images_per_row;

        let x = offset_x * self.image_dimensions.0;
        let y = offset_y * self.image_dimensions.1;

        let black_image: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> =
            Image::new(self.image_dimensions.0, self.image_dimensions.1);

        self.paste(&black_image, x, y);
    }
}
