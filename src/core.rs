use std::ops::{Deref, DerefMut};

use bytes::Bytes;
use image::{ImageBuffer, ImageFormat, Pixel, Rgb, Rgba};
use memmap::Mmap;

pub type RgbaImageBuffer<T> = ImageBuffer<Rgba<u8>, T>;
pub type RgbImageBuffer<T> = ImageBuffer<Rgb<u8>, T>;

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
