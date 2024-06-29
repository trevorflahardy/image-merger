use crate::BufferedImage;
use image::Pixel;

/// A trait that allows a Merger to resize images before pasting them onto the canvas. It allows
/// any existing merger to resize images before pasting. This is useful for when you have thousands of
/// images to paste, but you don't want to hold all of them, at full size, in memory at once with a large
/// canvas.
/// # Type Parameters
/// * `P` - The pixel type of the images to be resized.
/// # Example
/// ```
/// use image_merger::{ResizableMerger, KnownSizeMerger, Merger, Rgb};
///
/// // Create a canvas that can hold 100x100 images, with 10 images per row, and 100 images total.
/// let mut merger: KnownSizeMerger<Rgb<u8>, _> = KnownSizeMerger::new((100, 100), 10, 100, None);
///
/// // Push a 500x500 sized image onto the canvas. It will automatically be resized to 100x100.
/// merger.push_resized(&image_merger::Image::new(500, 500));
/// ```
pub trait ResizableMerger<P>
where
    P: Pixel + Sync,
    <P as Pixel>::Subpixel: Sync,
{
    /// Pushes an image onto the canvas after resizing it to the dimensions set on the merger.
    /// # Arguments
    /// * `image` - The image to push onto the canvas. Its pixel type, `P`, must match the canvas.
    fn push_resized(&mut self, image: &BufferedImage<P>);

    /// Bulk pushes N images onto the canvas after resizing them to the dimensions set on the merger.
    /// # Arguments
    /// * `images` - The images to push onto the canvas. Note that the argument type is `&[&Image<...>]`, the func does not need to take ownership of the images, it only needs to read them. The pixel type, `P`, of the images must match the canvas.
    fn bulk_push_resized(&mut self, images: &[&BufferedImage<P>]);
}
