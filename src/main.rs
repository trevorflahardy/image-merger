mod core;
mod merger;

use std::ops::Deref;

/// An image merger that allows for the merging of multiple images into one as a grid.
/// You can push a new image into the builder, append it to the main image, then drop it from memory to only have N images in memory at a time.
use merger::Merger;

fn generate_test_square() -> core::RgbaImageBuffer<Vec<u8>> {
    let random_rgba = |x: u8, y: u8| -> image::Rgba<u8> {
        // Generates random RGBA based on the x and y coordinates so that the square
        // is a gradient from red to green to blue.
        image::Rgba([x, y, 255 - x, 255])
    };

    let mut image = core::RgbaImageBuffer::new(100, 100);
    for x in 0..100 {
        for y in 0..100 {
            image.put_pixel(x, y, random_rgba(x as u8, y as u8));
        }
    }

    image
}

fn main() -> () {
    let mut merger: Merger<image::Rgba<u8>> = Merger::new((100, 100), 10);

    let image = core::Image::from(generate_test_square());

    let start_time = std::time::Instant::now();
    for _ in 0..100 {
        (&mut merger).push(&image);
        println!("Num images: {}", merger.get_num_images());
    }
    let end_time = std::time::Instant::now();
    println!(
        "Time to paste 1000 images auto adjusting canvas size 100 times: {:?}",
        end_time - start_time
    );

    // Save the image for testing
    merger.get_canvas().deref().save("test.png").unwrap();
}
