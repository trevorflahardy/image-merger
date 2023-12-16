use crate::{cell::ImageCell, core::Image, functions::paste};
use image::Pixel;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{marker::Sync, ops::DerefMut};

/// Represents a point on any canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

/// Represents the padding between images on a canvas.
/// # Fields
/// * `x` - The padding between images on the x axis.
/// * `y` - The padding between images on the y axis.
pub type Padding = Point;

/// The Merger trait that all mergers must implement. This trait allows the merger to paste images to a canvas.
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
pub trait Merger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    /// Returns a reference to the underlying canvas.
    fn get_canvas(&self) -> &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>>;

    /// Pushes a single image onto the canvas to be merged.
    /// # Arguments
    /// * `image` - The image to push onto the canvas.
    fn push<Container>(&mut self, image: &Image<P, image::ImageBuffer<P, Container>>) -> ()
    where
        Container: DerefMut<Target = [P::Subpixel]>;

    /// Pushes a vector of images onto the canvas to be merged.
    /// # Arguments
    /// * `images` - The images to push onto the canvas. Note that the argument type is `&Vec<&Image<...>>`, this is because the func
    /// does not need to take ownership of the images, it only needs to read them.
    fn bulk_push<Container>(
        &mut self,
        images: &Vec<&Image<P, image::ImageBuffer<P, Container>>>,
    ) -> ()
    where
        Container: DerefMut<Target = [P::Subpixel]> + Sync;
}

/// A fixed size merger that allows you to paste images onto a canvas. This merger is useful when you already know the size
/// of all the images being pushed onto the canvas.
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
pub struct FixedSizeMerger<P: Pixel> {
    canvas: ImageCell<P, image::ImageBuffer<P, Vec<P::Subpixel>>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted (images must be a uniform size)
    num_images: u32,              // The number of images that have been pasted to the canvas
    images_per_row: u32,          // The number of pages per row.
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    total_rows: u32,        // The total number of rows currently on the canvas.
    padding: Option<Padding>,
}

impl<P> FixedSizeMerger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    /// Constructs a new FixedSizeMerger.
    /// # Arguments
    /// * `image_dimensions` - The dimensions of the images being pasted (images must be a uniform size)
    /// * `images_per_row` - The number of images per row.
    /// * `rows` - The number of rows.
    /// * `padding` - The padding between images, or None for no padding.
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

    /// Returns the number of images that have been pasted to the canvas.
    pub fn get_num_images(&self) -> u32 {
        self.num_images
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

    /// Removes an image from the canvas at a given index. Indexing starts at 0 and works left to right, top to bottom.
    /// # Arguments
    /// * `index` - The index of the image to remove.
    pub fn remove_image(&mut self, index: u32) {
        let offset_x = index % self.images_per_row;
        let offset_y = index / self.images_per_row;

        let x = offset_x * self.image_dimensions.0;
        let y = offset_y * self.image_dimensions.1;

        let black_image: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> =
            Image::new(self.image_dimensions.0, self.image_dimensions.1);

        paste(&self.canvas, &black_image, Point { x, y });
    }
}

impl<P> Merger<P> for FixedSizeMerger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    fn get_canvas(&self) -> &Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> {
        &self.canvas
    }

    /// Allows the merger to push an image to the canvas. This can be used in a loop to paste a large number of images without
    /// having to hold all them in memory.
    fn push<Container>(&mut self, image: &Image<P, image::ImageBuffer<P, Container>>) -> ()
    where
        Container: DerefMut<Target = [P::Subpixel]>,
    {
        let (x, y) = self.get_next_paste_coordinates();

        paste(&self.canvas, image, Point { x, y });

        self.last_pasted_index += 1;
        self.num_images += 1;
    }

    /// Allows the merger to bulk push N images to the canvas. This is useful for when you have a large number of images to paste.
    /// The downside is that you have to hold all of the images in memory at once, which can be a problem if you have a large number of images.
    fn bulk_push<Container>(
        &mut self,
        images: &Vec<&Image<P, image::ImageBuffer<P, Container>>>,
    ) -> ()
    where
        Container: DerefMut<Target = [P::Subpixel]> + Sync,
    {
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
            paste(&self.canvas, image, Point { x, y });
        });

        self.last_pasted_index += images.len() as i32;
    }
}
