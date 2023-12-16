use std::ops::{Deref, DerefMut};

use image::{ImageBuffer, ImageFormat, Pixel, Rgb, Rgba};

pub type RgbaImageBuffer<Container> = ImageBuffer<Rgba<u8>, Container>;
pub type RgbImageBuffer<Container> = ImageBuffer<Rgb<u8>, Container>;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Image<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: U,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Image<P, U> {
    pub fn capacity(&self) -> usize {
        return self.underlying.pixels().count() * <P as Pixel>::CHANNEL_COUNT as usize;
    }
}

impl<P: Pixel> Image<P, ImageBuffer<P, Vec<P::Subpixel>>> {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            underlying: ImageBuffer::new(width, height),
        }
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

impl<P, Container> From<ImageBuffer<P, Container>> for Image<P, ImageBuffer<P, Container>>
where
    P: Pixel,
    Container: DerefMut<Target = [P::Subpixel]>,
{
    fn from(image: ImageBuffer<P, Container>) -> Self {
        Self { underlying: image }
    }
}

// TODO: Implement this
// pub trait FromWithFormat<T> {
//     fn from_with_format(t: T, format: ImageFormat) -> Self;
// }

// impl<Container, P> FromWithFormat<Container>
//     for Image<P, ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>>
// where
//     Container: Deref<Target = [u8]>,
//     P: Pixel,
// {
//     fn from_with_format(bytes: Container, format: ImageFormat) -> Self {
//         let dyn_image = image::load_from_memory_with_format(&bytes, format).unwrap();
//         let image: ImageBuffer<P, Vec<P::Subpixel>> = todo!();
//         Self { underlying: image }
//     }
// }
