use std::ops::{Deref, DerefMut};

use image::{ImageBuffer, ImageFormat, Luma, LumaA, Pixel, Rgb, Rgba};

/// Represents an image that can be passed to the merger. This is a wrapper around an image crate's GenericImage
/// and adds some additional functionality for the merger.
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
/// * `U` - The underlying image type.
/// # Example
/// ```
/// use image_merger::{Image, Rgba};
/// use image::ImageBuffer;
///
/// let image: Image<Rgba<u8>, ImageBuffer<Rgba<u8>, Vec<u8>>> = Image::new(100, 100);
/// assert_eq!(image.capacity(), 100 * 100 * 4);
/// ```
/// Note that this is for example, in practicality, you should use the [BufferedImage](crate::BufferedImage) type alias.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Image<P: Pixel, U: image::GenericImage<Pixel = P>> {
    underlying: U,
}

impl<P: Pixel, U: image::GenericImage<Pixel = P>> Image<P, U> {
    /// Returns the capacity of the underlying image's data buffer.
    pub fn capacity(&self) -> usize {
        return self.underlying.pixels().count() * <P as Pixel>::CHANNEL_COUNT as usize;
    }

    /// Consumes the image and returns the underlying image buffer.
    pub fn into_buffer(self) -> U {
        self.underlying
    }
}

impl<P, Container> Image<P, ImageBuffer<P, Container>>
where
    P: Pixel,
    Container: DerefMut<Target = [P::Subpixel]>,
{
    /// Creates a new image from a raw buffer. The buffer must be large enough to fit the image. Normally, you should use the
    /// `new` method to create a new image, as it is more idiomatic, unless you need to manually create an image from a raw buffer.
    ///
    /// # Arguments
    /// * `width` - The width of the image.
    /// * `height` - The height of the image.
    /// * `container` - The raw buffer to create the image from.
    ///
    /// # Returns
    /// An [Image](Image) with the given pixel and buffer type. Will return None if the buffer is not large enough to fit the image.
    pub fn new_from_raw(width: u32, height: u32, container: Container) -> Option<Self> {
        ImageBuffer::from_raw(width, height, container).map(|image| Self { underlying: image })
    }
}

impl<P: Pixel> Image<P, ImageBuffer<P, Vec<P::Subpixel>>> {
    /// Creates a new image with the given width and height.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            underlying: ImageBuffer::new(width, height),
        }
    }

    // Creates a new image from a given pixel, where the generated image will have the given width and height,
    // and the image will have the color of the pixel.
    pub fn new_from_pixel(width: u32, height: u32, pixel: P) -> Self {
        Self {
            underlying: ImageBuffer::from_pixel(width, height, pixel),
        }
    }
}

/// Dereferences to the underlying image.
///
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
/// * `U` - The underlying image type.
///
/// # Example
/// ```
/// use image_merger::{Image, Rgba, BufferedImage};
/// use image::ImageBuffer;
///
/// let image: BufferedImage<Rgba<u8>> = BufferedImage::new(100, 100);
/// let underlying: &ImageBuffer<Rgba<u8>, Vec<u8>> = &*image;
/// ```
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

/// A trait that allows the creation of an [Image](Image) from a preexisting [image::ImageBuffer](image::ImageBuffer).
///
/// # Type Parameters
/// * `P` - The pixel type of the underlying image.
/// * `Container` - The underlying image buffer type. This must be dereferenceable to a slice of the underlying image's subpixels.
///
/// # Example
/// ```
/// use image_merger::{Image, Rgba, BufferedImage};
/// use image::ImageBuffer;
///
/// let buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(100, 100);
/// let image: BufferedImage<Rgba<u8>> = BufferedImage::from(buf);
/// ```
impl<P, Container> From<ImageBuffer<P, Container>> for Image<P, ImageBuffer<P, Container>>
where
    P: Pixel,
    Container: DerefMut<Target = [P::Subpixel]>,
{
    /// Creates a new Image from a preexisting ImageBuffer.
    /// # Arguments
    /// * `image` - The ImageBuffer to create an Image from.
    fn from(image: ImageBuffer<P, Container>) -> Self {
        Self { underlying: image }
    }
}

/// A trait that allows the creation of an Image from a container of bytes using a specified image format.
/// # Type Parameters
/// * `Container` - The container type. This must be dereferenceable to a slice of bytes.
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
    /// # Panics
    /// This function will panic if the given container cannot be transformed into an image with the given format.
    /// # Example
    /// ```no_run
    /// use image_merger::{FromWithFormat, Image, Rgba, BufferedImage};
    /// use image::ImageBuffer;
    ///
    /// let container = vec![0, 0, 0, 255, 255, 255, 255, 255];
    /// let image: BufferedImage<Rgba<u8>> = BufferedImage::from_with_format(container, image::ImageFormat::Png);
    /// ```
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
