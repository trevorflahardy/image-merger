extern crate image_merger;

use image_merger::*;

const IMAGE_WIDTH: u32 = 100;
const IMAGE_HEIGHT: u32 = 100;
const IMAGES_PER_ROW: u32 = 10;
const TOTAL_IMAGES: u32 = 100;

fn generate_known_image() -> BufferedImage<Rgba<u8>> {
    // Create an image buffer with the given dimensions
    BufferedImage::new_from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, Rgba([255, 0, 0, 255]))
}

fn main() -> () {
    // Generate a image we can paste to our canvas. In a real application, this may be an opened
    // image file or buffer of some sort. For the sake of example, the constants IMAGE_WIDTH and IMAGE_HEIGHT
    // will represent our known image dimensions.
    let known_image = generate_known_image();

    // Create an instance of our merger, this is what will manage the merging of our images. It takes one generic
    // parameter, T, which denotes the type of pixel the canvas and pasted images have.
    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );

    // Let's go through and paste our images onto the canvas, we can do this one of two ways:
    // 1. We can paste the images one at a time in a loop, using the "push()" method, or:
    // 2: We can use the "bulk_push()" method to paste multiple images at once.
    // For this example, we'll use the "bulk_push()" method.
    let images: Vec<&BufferedImage<Rgba<u8>>> = vec![&known_image; TOTAL_IMAGES as usize];
    merger.bulk_push(&images);

    // Finally, we can get the canvas and save it - we should have a red image with 10000 pixels.
    let canvas = merger.get_canvas();
    canvas.save("examples/known_size_merger.png").unwrap();
}
