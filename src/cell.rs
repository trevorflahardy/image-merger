use super::core::Image;
use image::Pixel;
use std::{
    cell::UnsafeCell,
    collections::HashSet,
    marker::{Send, Sync},
    ops::Deref,
};

// A struct that allows multible mutable references to an underlying image's data buffer
// without violating Rust's safety rules.
pub struct ImageCell<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: UnsafeCell<Image<P, U>>,
    handouts: UnsafeCell<HashSet<(u32, u32)>>,
}

pub struct Handout<'a, P: Pixel, U: image::GenericImage<Pixel = P>> {
    ic: &'a ImageCell<P, U>,
    x: u32,
    y: u32,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> ImageCell<P, U> {
    pub fn new(image: Image<P, U>) -> Self {
        let capacity = image.width() * image.height();
        Self {
            underlying: UnsafeCell::new(image),
            handouts: UnsafeCell::new(HashSet::with_capacity(capacity as usize)),
        }
    }

    pub fn into_inner(self) -> Image<P, U> {
        self.underlying.into_inner()
    }

    pub(crate) fn get_image_mut(&self) -> &mut Image<P, U> {
        unsafe { &mut *self.underlying.get() }
    }

    fn get_handouts_mut(&self) -> &mut HashSet<(u32, u32)> {
        unsafe { &mut *self.handouts.get() }
    }

    pub unsafe fn request_handout_unchecked(&self, x: u32, y: u32) -> Handout<P, U> {
        Handout { ic: &self, x, y }
    }

    pub fn request_handout(&self, x: u32, y: u32) -> Option<Handout<P, U>> {
        let handouts = self.get_handouts_mut();

        if handouts.get(&(x, y)).is_some() {
            // Handout already requested for this pixel, need to return None in this case
            return None;
        }

        handouts.insert((x, y));

        unsafe { Some(self.request_handout_unchecked(x, y)) }
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
    pub fn put_pixel(&mut self, pixel: P) {
        let image = self.ic.get_image_mut();
        image.put_pixel(self.x, self.y, pixel);
    }

    pub unsafe fn unsafe_put_pixel(&mut self, pixel: P) {
        let image = self.ic.get_image_mut();
        image.unsafe_put_pixel(self.x, self.y, pixel);
    }
}
