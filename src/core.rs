use std::ops::{Deref, DerefMut};

use image::{ImageBuffer, ImageFormat, Luma, LumaA, Pixel, Rgb, Rgba};

/// Represents an image that can be passed to the merger. This is a wrapper around an image crate's GenericImage
/// and adds some additional functionality for the merger.
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
/// * `U` - The underlying image type.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Image<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: U,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Image<P, U> {
    /// Returns the capacity of the underlying image's data buffer.
    pub fn capacity(&self) -> usize {
        return self.underlying.pixels().count() * <P as Pixel>::CHANNEL_COUNT as usize;
    }
}

impl<P: Pixel> Image<P, ImageBuffer<P, Vec<P::Subpixel>>> {
    /// Creates a new image with the given width and height.
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
    /// Creates a new Image from a preexisting ImageBuffer.
    fn from(image: ImageBuffer<P, Container>) -> Self {
        Self { underlying: image }
    }
}

/// A trait that allows the creation of an Image from a container of bytes using a specified image format.
/// # Type Parameters
/// * `T` - The container type. This must be dereferenceable to a slice of bytes.
pub trait FromWithFormat<Container>
where
    Container: Deref<Target = [u8]>,
{
    /// Transforms the given container and image format into an Image.
    /// # Arguments
    /// * `container` - The container to transform into an Image.
    /// * `format` - The format of the image.
    /// # Returns
    /// An [Image](Image) with the given pixel and buffer type.
    fn from_with_format(container: Container, format: ImageFormat) -> Self;
}

macro_rules! impl_from_with_format {
    ($px_type:ident, $channel_type:ty, $to_fn:ident) => {
        #[doc = concat!(
            r#"Implementation of [`FromWithFormat`](FromWithFormat) for an [`Image`](Image) with a pixel type of [`"#,
            stringify!($px_type),
            "`](image::",
            stringify!($px_type),
            "), holding a subpixel type of [`",
            stringify!($channel_type),
            r#"`]("#
            , stringify!($channel_type),
            r#") and an underlying [`ImageBuffer`](image::ImageBuffer) buffer that holds `Vec<"#,
            stringify!($channel_type),
            r#">`'s.
        "#)]
        impl<Container> FromWithFormat<Container>
            for Image<
                $px_type<$channel_type>,
                ImageBuffer<$px_type<$channel_type>, Vec<$channel_type>>,
            >
        where
            Container: Deref<Target = [u8]>,
        {
            fn from_with_format(container: Container, format: ImageFormat) -> Self {
                let dyn_image = image::load_from_memory_with_format(&container, format).unwrap();
                let img = dyn_image.$to_fn();

                Self::from(img)
            }
        }
    };
}

impl_from_with_format!(Rgb, u8, into_rgb8);
impl_from_with_format!(Rgb, u16, into_rgb16);
impl_from_with_format!(Rgb, f32, into_rgb32f);

impl_from_with_format!(Rgba, u8, into_rgba8);
impl_from_with_format!(Rgba, u16, into_rgba16);
impl_from_with_format!(Rgba, f32, into_rgba32f);

impl_from_with_format!(Luma, u8, into_luma8);
impl_from_with_format!(Luma, u16, into_luma16);

impl_from_with_format!(LumaA, u8, into_luma_alpha8);
impl_from_with_format!(LumaA, u16, into_luma_alpha16);
