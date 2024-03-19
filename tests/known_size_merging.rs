use image::imageops::overlay;
use image_merger::*;

static IMAGES_PER_ROW: u32 = 10;
static TOTAL_ROWS: u32 = 10;
static TOTAL_IMAGES: u32 = 100;
static PADDING_X: u32 = 10;
static PADDING_Y: u32 = 10;
static IMAGE_WIDTH: u32 = 100;
static IMAGE_HEIGHT: u32 = 100;

type RgbaImageBuffer = BufferedImage<Rgba<u8>>;

// TODO: Move this to a test utils file, make it generic, and take in an image index for unique colors
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
        IMAGE_WIDTH * images_per_row + (padding_x * (images_per_row - 1)),
        IMAGE_HEIGHT * total_rows + (padding_y * (total_rows - 1)),
    );

    for index in 0..total_images {
        let global_x = index % images_per_row;
        let global_y = index / images_per_row;

        let x = (global_x * IMAGE_WIDTH) + (global_x * padding_x);
        let y = (global_y * IMAGE_HEIGHT) + (global_y * padding_y);

        overlay(&mut *canvas, &*test_square, x as i64, y as i64)
    }

    canvas
}

#[test]
fn test_slow_merge() {
    let merged = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, 0, 0);
    assert_eq!(merged.width(), IMAGES_PER_ROW * IMAGE_WIDTH);
    assert_eq!(merged.height(), IMAGES_PER_ROW * IMAGE_HEIGHT);
}

#[test]
fn test_slow_merge_padding() {
    let merged = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, PADDING_X, PADDING_Y);
    assert_eq!(
        merged.width(),
        IMAGES_PER_ROW * IMAGE_WIDTH + (PADDING_X * (IMAGES_PER_ROW - 1))
    );
    assert_eq!(
        merged.height(),
        TOTAL_ROWS * IMAGE_HEIGHT + (PADDING_Y * (TOTAL_ROWS - 1))
    );
}

#[test]
fn test_push_merge() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, 0, 0);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        None,
    );

    for _ in 0..TOTAL_IMAGES {
        merger.push(&test_square);
    }

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
#[should_panic(expected = "No more space on the canvas.")]
fn test_push_merge_fail() {
    let test_square = generate_test_square();

    let mut merger: KnownSizeMerger<Rgba<u8>> =
        KnownSizeMerger::new((IMAGE_WIDTH, IMAGE_HEIGHT), 1, 1, None);

    for _ in 0..2 {
        merger.push(&test_square);
    }
}

#[test]
fn test_bulk_push_merge() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, 0, 0);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        None,
    );
    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_push_merge_padding() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, PADDING_X, PADDING_Y);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        Some(Padding {
            x: PADDING_X,
            y: PADDING_Y,
        }),
    );

    for _ in 0..TOTAL_IMAGES {
        merger.push(&test_square);
    }

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_bulk_push_merge_padding() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, PADDING_X, PADDING_Y);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        Some(Padding {
            x: PADDING_X,
            y: PADDING_Y,
        }),
    );
    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_remove_image() {
    // 99 images on the slow merge should be equal to 100 images on the fast merge minus the 1 removed image.
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES - 1, 0, 0);

    let mut merger: KnownSizeMerger<Rgba<u8>> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_ROWS,
        None,
    );

    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);
    merger.remove_image(99);

    assert_eq!(merger.get_canvas(), &slow_merge);
}
