use super::core::{Merger, MergerInfo};
use crate::{core::Image, functions::resize_nearest_neighbor};
use image::Pixel;
use std::ops::DerefMut;

/// A trait that allows mergers to resize images before pushing them onto the canvas. This is auto implemented
/// for every merger that implements `Merger` and `MergerInfo`.
pub trait ResizeMerger<P>: Merger<P> + MergerInfo
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    /// Resizes the given image and pushes it into the merger.
    ///
    /// # Arguments
    ///
    /// * `image` - The image to be resized and pushed.
    ///
    /// # Generic Parameters
    ///
    /// * `Container` - The container type for the image's pixel data.
    ///
    /// # Constraints
    ///
    /// The `Container` type must implement `DerefMut` with a target of `[P::Subpixel]`.
    ///
    fn resize_push<Container>(&mut self, image: &Image<P, image::ImageBuffer<P, Container>>)
    where
        Container: DerefMut<Target = [P::Subpixel]>,
    {
        let (width, height) = self.get_image_dimensions();
        let (img_width, img_height) = image.dimensions();
        if (img_width, img_height) != (width, height) {
            let resized = resize_nearest_neighbor(image, width, height);
            self.push(&resized);
        } else {
            self.push(image);
        }
    }

    /// Resizes the given images and bulk pushes them into the merger. Unfortunately, this method
    /// requires the ownership of the images, unlike `bulk_push`, which only requires a reference. This
    /// will be fixed in the future.
    ///
    /// # Arguments
    ///
    /// * `images` - The vector of images to be resized and pushed.
    ///
    /// # Generic Parameters
    ///
    /// * `Container` - The container type for the images' pixel data.
    ///
    /// # Constraints
    ///
    /// The `Container` type must implement `DerefMut` with a target of `[P::Subpixel]` and `Sync`.
    ///
    fn resize_bulk_push<Container>(
        &mut self,
        images: Vec<Image<P, image::ImageBuffer<P, Container>>>,
    ) where
        Container: DerefMut<Target = [P::Subpixel]> + Sync,
    {
        // TODO: Update this method to take a slice of references instead of a vector of owned images.

        let (width, height) = self.get_image_dimensions();
        let mut resized_images: Vec<Image<P, image::ImageBuffer<P, Container>>> =
            Vec::with_capacity(images.len());

        for image in images.into_iter() {
            let (img_width, img_height) = image.dimensions();
            if (img_width, img_height) != (width, height) {
                let resized = resize_nearest_neighbor(&image, width, height);
                resized_images.push(resized);
            } else {
                resized_images.push(image);
            }
        }

        let resized_images_ref: Vec<&Image<P, image::ImageBuffer<P, Container>>> =
            resized_images.iter().collect();

        self.bulk_push(&resized_images_ref);
    }
}

/// Auto implementation of `ResizeMerger` for every merger that implements `Merger` and `MergerInfo`.
/// This is done so that every merger can resize images before pushing them onto the canvas.
impl<P, T> ResizeMerger<P> for T
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
    T: Merger<P> + MergerInfo,
{
}
