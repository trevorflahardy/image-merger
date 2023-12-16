use super::core::Image;
use image::Pixel;
use std::{
    cell::UnsafeCell,
    marker::{Send, Sync},
    ops::Deref,
};

/// A struct that allows multible mutable references to an underlying image's data buffer. This is an
/// unsafe struct and should only be used when no two items are trying to change the same place in the underlying
/// image's data buffer. This struct is used to allow multible threads to write to the same image at the same time.
pub struct ImageCell<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: UnsafeCell<Image<P, U>>,
}

pub struct Handout<'a, P: Pixel, U: image::GenericImage<Pixel = P>> {
    ic: &'a ImageCell<P, U>,
    x: u32,
    y: u32,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> ImageCell<P, U> {
    pub fn new(image: Image<P, U>) -> Self {
        Self {
            underlying: UnsafeCell::new(image),
        }
    }

    /// Returns the underlying image.
    pub fn into_inner(self) -> Image<P, U> {
        self.underlying.into_inner()
    }

    pub(crate) fn get_image_mut(&self) -> &mut Image<P, U> {
        unsafe { &mut *self.underlying.get() }
    }

    /// Requests a handout at the given coordinates of the undelrying image. Can be be used to write
    /// to an underlying image buffer across threads without a mutable reference to the underlying image.
    /// # Safety
    /// This function is unsafe because it does not implement any thread safety via locks or anything else. It is up to the caller to ensure that
    /// no two threads are trying to write to the same place in the underlying image's data buffer.
    ///
    /// # Arguments
    /// * `x` - The x coordinate of the pixel to request a handout for.
    /// * `y` - The y coordinate of the pixel to request a handout for.
    /// # Returns
    /// A handout that can be used to write to the underlying image's data buffer.
    /// # Example
    /// ```
    /// let image_cell: ImageCell<P, _> = ...;
    /// let mut handout = unsafe { image_cell.request_handout(0, 0) };
    /// handout.put_pixel(...);
    pub unsafe fn request_handout(&self, x: u32, y: u32) -> Handout<P, U> {
        Handout { ic: &self, x, y }
    }
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Deref for ImageCell<P, U> {
    type Target = Image<P, U>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.underlying.get() }
    }
}

unsafe impl<P: Pixel, U: image::GenericImage<Pixel = P>> Sync for ImageCell<P, U> {}
unsafe impl<P: Pixel, U: image::GenericImage<Pixel = P>> Send for ImageCell<P, U> {}

impl<'a, P: Pixel, U: image::GenericImage<Pixel = P>> Handout<'a, P, U> {
    /// Puts a pixel at the handout's coordinates.
    pub fn put_pixel(&mut self, pixel: P) {
        let image = self.ic.get_image_mut();
        image.put_pixel(self.x, self.y, pixel);
    }

    /// Same as `put_pixel` but does not check bounds.
    pub unsafe fn unsafe_put_pixel(&mut self, pixel: P) {
        let image = self.ic.get_image_mut();
        image.unsafe_put_pixel(self.x, self.y, pixel);
    }
}
