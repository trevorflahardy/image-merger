use image::imageops::overlay;
use image_merger::*;

static IMAGES_PER_ROW: u32 = 10;
static TOTAL_ROWS: u32 = 10;
static TOTAL_IMAGES: u32 = 100;

static IMAGE_WIDTH: u32 = 200;
static IMAGE_HEIGHT: u32 = 200;

static MERGER_WIDTH: u32 = 100;
static MERGER_HEIGHT: u32 = 100;

type RgbaImageBuffer = BufferedImage<Rgba<u8>>;

// TODO: Move this to a test utils file (?), make it generic, and take in an image index for unique colors
// across an entire image.
fn generate_test_square() -> RgbaImageBuffer {
    let color = |x: u32, y: u32| -> Rgba<u8> {
        let r = x as u8;
        let g = y as u8;
        let b = (x + y) as u8;
        Rgba([r, g, b, 255])
    };

    let mut image = RgbaImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    for x in 0..IMAGE_WIDTH {
        for y in 0..IMAGE_HEIGHT {
            image.put_pixel(x, y, color(x, y));
        }
    }

    image
}

fn merge_images_slow(
    images_per_row: u32,
    total_images: u32,
    padding_x: u32,
    padding_y: u32,
) -> RgbaImageBuffer {
    // Cieling division for total rows.
    let total_rows = (total_images + images_per_row - 1) / images_per_row;

    let test_square = generate_test_square();

    let mut canvas = RgbaImageBuffer::new(
        MERGER_WIDTH * images_per_row + (padding_x * (images_per_row - 1)),
        MERGER_HEIGHT * total_rows + (padding_y * (total_rows - 1)),
    );

    for index in 0..total_images {
        let global_x = index % images_per_row;
        let global_y = index / images_per_row;

        let x = (global_x * MERGER_WIDTH) + (global_x * padding_x);
        let y = (global_y * MERGER_HEIGHT) + (global_y * padding_y);

        overlay(&mut *canvas, &*test_square, x as i64, y as i64)
    }

    canvas
}

#[test]
fn test_resize_push() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, 0, 0);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (MERGER_WIDTH, MERGER_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        Some(Point { x: 10, y: 10 }),
    );

    for _ in 0..TOTAL_IMAGES {
        merger.resize_push(&test_square);
    }

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_resize_bulk_push_merge() {
    let mut images: Vec<RgbaImageBuffer> = Vec::with_capacity(TOTAL_IMAGES as usize);
    for _ in 0..TOTAL_IMAGES {
        let test_square = generate_test_square();
        images.push(test_square);
    }

    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, 0, 0);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        None,
    );
    merger.resize_bulk_push(images);

    assert_eq!(merger.get_canvas(), &slow_merge);
}
