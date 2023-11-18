use bytes::Bytes;
use image::{buffer::ConvertBuffer, ImageBuffer, ImageFormat, Pixel, Rgb, Rgba};
use memmap::Mmap;

pub(crate) type RgbaImageBuffer<T> = ImageBuffer<Rgba<u8>, T>;
pub(crate) type RgbImageBuffer<T> = ImageBuffer<Rgb<u8>, T>;

pub trait FromWithFormat<T> {
    fn from_with_format(t: T, format: ImageFormat) -> Self;
}

pub struct Image<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: U,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Image<P, U> {
    pub fn get_underlying(&self) -> &U {
        return &self.underlying;
    }
}

impl From<Bytes> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from(bytes: Bytes) -> Self {
        Self {
            underlying: image::load_from_memory(bytes.as_ref()).unwrap().to_rgba8(),
        }
    }
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

impl From<Mmap> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from(mmap: Mmap) -> Self {
        Self {
            underlying: image::load_from_memory(&mmap).unwrap().to_rgba8(),
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

impl From<RgbaImageBuffer<Vec<u8>>> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from(rgba_image: RgbaImageBuffer<Vec<u8>>) -> Self {
        Self {
            underlying: rgba_image,
        }
    }
}

impl From<RgbImageBuffer<Vec<u8>>> for Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    fn from(rgb_image: RgbImageBuffer<Vec<u8>>) -> Self {
        Self {
            underlying: rgb_image.convert(),
        }
    }
}
