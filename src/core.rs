use std::{
    cell::UnsafeCell,
    marker::{Send, Sync},
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use image::{ImageBuffer, ImageFormat, Pixel, Rgb, Rgba};
use memmap::Mmap;

pub(crate) type RgbaImageBuffer<T> = ImageBuffer<Rgba<u8>, T>;
pub(crate) type RgbImageBuffer<T> = ImageBuffer<Rgb<u8>, T>;

pub struct Image<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: U,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Image<P, U> {
    pub fn capacity(&self) -> usize {
        return self.underlying.pixels().count() * <P as Pixel>::CHANNEL_COUNT as usize;
    }
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Deref for Image<P, U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.underlying
    }
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> DerefMut for Image<P, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.underlying
    }
}

impl From<Bytes> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from(bytes: Bytes) -> Self {
        Self {
            underlying: image::load_from_memory(bytes.as_ref()).unwrap().to_rgba8(),
        }
    }
}

impl From<Mmap> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from(mmap: Mmap) -> Self {
        Self {
            underlying: image::load_from_memory(&mmap).unwrap().to_rgba8(),
        }
    }
}

impl<P: Pixel> From<ImageBuffer<P, Vec<P::Subpixel>>>
    for Image<P, ImageBuffer<P, Vec<P::Subpixel>>>
{
    fn from(image: ImageBuffer<P, Vec<P::Subpixel>>) -> Self {
        Self { underlying: image }
    }
}

pub trait FromWithFormat<T> {
    fn from_with_format(t: T, format: ImageFormat) -> Self;
}

impl FromWithFormat<Bytes> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from_with_format(bytes: Bytes, format: ImageFormat) -> Self {
        Self {
            underlying: image::load_from_memory_with_format(bytes.as_ref(), format)
                .unwrap()
                .to_rgba8(),
        }
    }
}

impl FromWithFormat<Mmap> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from_with_format(mmap: Mmap, format: ImageFormat) -> Self {
        Self {
            underlying: image::load_from_memory_with_format(&mmap, format)
                .unwrap()
                .to_rgba8(),
        }
    }
}

// A struct that allows multible mutable references to an underlying image's data buffer
// without violating Rust's safety rules.
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

    pub(crate) fn get_image_mut(&self) -> &mut Image<P, U> {
        unsafe { &mut *self.underlying.get() }
    }

    // SAFETY: This function must be called only once per thread with the assumption that
    // no other thread will access the same x, y pixel coordinates (creating a race condition).
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

unsafe impl<'a, P: Pixel, U: image::GenericImage<Pixel = P>> Sync for ImageCell<P, U> {}
unsafe impl<'a, P: Pixel, U: image::GenericImage<Pixel = P>> Send for ImageCell<P, U> {}

impl<'a, P: Pixel, U: image::GenericImage<Pixel = P>> Handout<'a, P, U> {
    pub fn put_pixel(&mut self, pixel: P) {
        let image = self.ic.get_image_mut();
        image.put_pixel(self.x, self.y, pixel);
    }
}
