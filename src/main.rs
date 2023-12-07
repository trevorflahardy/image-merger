mod core;
mod merger;

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

fn perform_pasting(merger: &mut Merger<image::Rgba<u8>>, count: usize) {
    let image = core::Image::from(generate_test_square());

    (0..count).for_each(|_| {
        merger.push(&image);
    });
}
fn main() -> () {
    let mut merger: Merger<image::Rgba<u8>> = Merger::new((100, 100), 100, 100);

    let start_time = std::time::Instant::now();

    perform_pasting(&mut merger, 1000);

    let end_time = std::time::Instant::now();
    println!(
        "Time to paste 1000 images auto adjusting canvas size 100 times: {:?}",
        end_time - start_time
    );

    let canvas = merger.get_canvas();
    canvas.save("test.png").expect("Failed to save image");
}
