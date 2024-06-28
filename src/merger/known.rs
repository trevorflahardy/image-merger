use super::core::{Merger, Padding, Point};
use crate::{cell::ImageCell, functions::paste, Image};

use image::Pixel;
use num_traits::Zero;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::ops::DerefMut;

/// A known size merger that allows you to paste images onto a canvas. This merger is useful when you already know the size
/// of all the images being pushed onto the canvas. This merger has multiple implementations, one for any container type and
/// one for Vec specifically.
///
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
/// * `Container` - The underlying image buffer type. This must be dereferenceable to a slice of the underlying image's subpixels.
///
/// # Example
/// ```
/// use image_merger::{Merger, KnownSizeMerger, Image, Rgb};
///
/// let mut merger: KnownSizeMerger<Rgb<u8>, _> = KnownSizeMerger::new((100, 100), 5, 10, None);
/// let image = Image::new(100, 100);
/// merger.bulk_push(&[&image, &image, &image, &image, &image]);
/// ```
pub struct KnownSizeMerger<P, Container>
where
    P: Pixel,
    <P as Pixel>::Subpixel: Sync,
    Container: DerefMut<Target = [P::Subpixel]> + Sync,
{
    canvas: ImageCell<P, image::ImageBuffer<P, Container>>,
    image_dimensions: (u32, u32), // The dimensions of the images being pasted (images must be a uniform size)
    num_images: u32,              // The number of images that have been pasted to the canvas
    images_per_row: u32,          // The number of pages per row.
    last_pasted_index: i32, // The index of the last pasted image, starts at -1 if not images have been pasted.
    total_rows: u32,        // The total number of rows currently on the canvas.
    padding: Option<Padding>,
}

impl<P, Container> KnownSizeMerger<P, Container>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    Container: DerefMut<Target = [P::Subpixel]> + Sync,
{
    /// Constructs a new KnownSizeMerger from a raw image buffer. This is useful if you need to use a specific container type that is not Vec. Typically,
    /// you only need to use `new` to create a KnownSizeMerger unless you need to use a specific container type.
    /// # Arguments
    /// * `image_dimensions` - The dimensions of the images being pasted (images must be a uniform size)
    /// * `images_per_row` - The number of images per row.
    /// * `total_images` - The total number of images to be in the final canvas.
    /// * `padding` - The padding between images, or None for no padding.
    /// * `container` - The container to use for the underlying canvas. This container must be big enough to hold all the potential images
    /// that will be pasted to the canvas.
    ///
    /// # Returns
    /// * `Some` - If the merger was successfully created.
    /// * `None` - If the merger could not be created. This will happen if the container is not large enough to fit all the images.
    ///
    /// # Example
    /// ```
    /// use image_merger::{KnownSizeMerger, Rgb};
    ///
    /// let container = vec![0 as u8; 100 * 100 * 5 * 10 * 3];
    /// let merger: KnownSizeMerger<Rgb<u8>, _> = KnownSizeMerger::new_from_raw((100, 100), 5, 10, None, container).expect("Could not create merger!");
    /// ```
    pub fn new_from_raw(
        image_dimensions: (u32, u32),
        images_per_row: u32,
        total_images: u32,
        padding: Option<Padding>,
        container: Container,
    ) -> Option<Self> {
        let total_rows = (total_images + images_per_row - 1) / images_per_row;

        let image_gaps_x = (images_per_row - 1) * padding.as_ref().map(|p| p.x).unwrap_or(0);
        let image_gaps_y = (total_rows - 1) * padding.as_ref().map(|p| p.y).unwrap_or(0);

        Image::new_from_raw(
            (image_dimensions.0 * images_per_row) + image_gaps_x,
            (image_dimensions.1 * total_rows) + image_gaps_y,
            container,
        )
        .map(|canvas| Self {
            canvas: ImageCell::new(canvas),
            image_dimensions,
            num_images: 0,
            images_per_row,
            last_pasted_index: -1,
            total_rows,
            padding,
        })
    }

    /// Returns the number of images that have been pasted to the canvas.
    pub fn get_num_images(&self) -> u32 {
        self.num_images
    }

    /// Returns the dimensions, (x, y), of the images being pasted to the canvas.
    pub fn get_image_dimensions(&self) -> (u32, u32) {
        self.image_dimensions
    }

    #[inline(always)]
    fn additional_space(&self) -> u32 {
        (self.images_per_row * self.total_rows) - self.num_images
    }

    fn get_paste_coordinates_unchecked(&self, index: u32) -> (u32, u32) {
        let offset_x = index % self.images_per_row;
        let offset_y = index / self.images_per_row;

        let padding_x = self.padding.as_ref().map(|p| p.x).unwrap_or(0) * offset_x;
        let padding_y = self.padding.as_ref().map(|p| p.y).unwrap_or(0) * offset_y;

        let x = (offset_x * self.image_dimensions.0) + padding_x;
        let y = (offset_y * self.image_dimensions.1) + padding_y;

        (x, y)
    }

    fn get_next_paste_coordinates(&mut self) -> (u32, u32) {
        if self.additional_space() == 0 {
            panic!("No more space on the canvas!");
        }

        self.get_paste_coordinates_unchecked((self.last_pasted_index + 1) as u32)
    }

    /// Removes an image from the canvas at the given index. Indices start at 0 and work left to right, top to bottom. Most of the time
    /// you will not need to use this function, and rather, can use the `remove_image` method instead. This method is useful if you need
    /// to manually manage a specific Container type that is not `Vec`.
    ///
    /// # Arguments
    /// * `index` - The index of the image to remove.
    /// * `container` - The container to use to replace the image. The container must be the same size as the image being removed,
    /// thus, the container must be the same size as the image dimensions.
    ///
    /// # Returns
    /// * `Some` - If the image was successfully removed.
    /// * `None` - If the image could not be removed. This will happen if the container is not large enough to fit the image.
    pub fn remove_image_raw(&mut self, index: u32, container: Container) -> Option<()> {
        let offset_x = index % self.images_per_row;
        let offset_y = index / self.images_per_row;

        let x = offset_x * self.image_dimensions.0;
        let y = offset_y * self.image_dimensions.1;

        let black_image =
            Image::new_from_raw(self.image_dimensions.0, self.image_dimensions.1, container);

        if let Some(black_image) = black_image {
            paste(&self.canvas, &black_image, Point { x, y });
            Some(())
        } else {
            None
        }
    }
}

impl<P> KnownSizeMerger<P, Vec<P::Subpixel>>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    /// Constructs a new KnownSizeMerger. By default, this will create an underlying canvas with `Vec` as the container type. If you
    /// need to use a different container type, you can use the `new_from_raw` method.
    ///
    /// # Arguments
    /// * `image_dimensions` - The dimensions of the images being pasted (images must be a uniform size)
    /// * `images_per_row` - The number of images per row.
    /// * `total_images` - The total numbr of images to be in the final canvas.
    /// * `padding` - The padding between images, or None for no padding.
    pub fn new(
        image_dimensions: (u32, u32),
        images_per_row: u32,
        total_images: u32,
        padding: Option<Padding>,
    ) -> Self {
        let total_rows = (total_images + images_per_row - 1) / images_per_row;

        let image_gaps_x = (images_per_row - 1) * padding.as_ref().map(|p| p.x).unwrap_or(0);
        let image_gaps_y = (total_rows - 1) * padding.as_ref().map(|p| p.y).unwrap_or(0);

        let canvas = Image::new(
            (image_dimensions.0 * images_per_row) + image_gaps_x,
            (image_dimensions.1 * total_rows) + image_gaps_y,
        );

        Self {
            canvas: ImageCell::new(canvas),
            image_dimensions,
            num_images: 0,
            images_per_row,
            last_pasted_index: -1,
            total_rows,
            padding,
        }
    }

    /// Removes an image from the canvas at a given index. Indexing starts at 0 and works left to right, top to bottom.
    /// # Arguments
    /// * `index` - The index of the image to remove.
    pub fn remove_image(&mut self, index: u32) {
        let container: Vec<<P as Pixel>::Subpixel> = vec![
            Zero::zero();
            (self.image_dimensions.0 * self.image_dimensions.1 * <P as Pixel>::CHANNEL_COUNT as u32)
                as usize
        ];

        self.remove_image_raw(index, container).unwrap(); // Can always unwrap here because we know the buffer is the right size.

        let offset_x = index % self.images_per_row;
        let offset_y = index / self.images_per_row;

        let x = offset_x * self.image_dimensions.0;
        let y = offset_y * self.image_dimensions.1;

        let black_image: Image<P, image::ImageBuffer<P, Vec<P::Subpixel>>> =
            Image::new(self.image_dimensions.0, self.image_dimensions.1);

        paste(&self.canvas, &black_image, Point { x, y });
    }
}

impl<P, Container> Merger<P, Container> for KnownSizeMerger<P, Container>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    Container: DerefMut<Target = [P::Subpixel]> + Sync,
{
    fn get_canvas(&self) -> &Image<P, image::ImageBuffer<P, Container>> {
        &self.canvas
    }

    fn into_canvas(self) -> Image<P, image::ImageBuffer<P, Container>> {
        self.canvas.into_inner()
    }

    fn push(&mut self, image: &Image<P, image::ImageBuffer<P, Container>>) {
        let (x, y) = self.get_next_paste_coordinates();

        paste(&self.canvas, image, Point { x, y });

        self.last_pasted_index += 1;
        self.num_images += 1;
    }

    fn bulk_push(&mut self, images: &[&Image<P, image::ImageBuffer<P, Container>>]) {
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
        self.num_images += images.len() as u32;
    }
}
