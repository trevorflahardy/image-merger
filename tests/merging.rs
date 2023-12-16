use image::{imageops::overlay, ImageBuffer, Rgba};
use image_merger::*;

static IMAGES_PER_ROW: u32 = 10;
static TOTAL_ROWS: u32 = 10;
static TOTAL_IMAGES: u32 = 100;
static PADDING_X: u32 = 10;
static PADDING_Y: u32 = 10;

type RgbaImageBuffer<Container> = ImageBuffer<Rgba<u8>, Container>;

fn generate_test_square() -> Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    let rgba = |x: u8, y: u8| -> image::Rgba<u8> {
        // Generates a gradient of RGBA based on the x and y coordinates.
        image::Rgba([
            255 - x,
            255 - y,
            ((255 * 2) as u32 - (x + y) as u32) as u8,
            255,
        ])
    };

    let mut image = Image::<Rgba<u8>, RgbaImageBuffer<Vec<u8>>>::new(100, 100);
    for x in 0..100 {
        for y in 0..100 {
            image.put_pixel(x, y, rgba(x as u8, y as u8));
        }
    }

    image
}

fn merge_images_slow(
    images_per_row: u32,
    total_images: u32,
    padding_x: u32,
    padding_y: u32,
) -> Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> {
    // Cieling division for total rows.
    let total_rows = (total_images + images_per_row - 1) / images_per_row;

    let test_square = generate_test_square();
    let test_square_width = test_square.width();
    let test_square_height = test_square.height();

    let mut canvas = Image::<Rgba<u8>, RgbaImageBuffer<Vec<u8>>>::new(
        test_square_width * images_per_row + (padding_x * (images_per_row - 1)),
        test_square_height * total_rows + (padding_y * (total_rows - 1)),
    );

    for index in 0..total_images {
        let global_x = index % images_per_row;
        let global_y = index / images_per_row;

        let x = (global_x * test_square_width) + (global_x * padding_x);
        let y = (global_y * test_square_height) + (global_y * padding_y);

        overlay(&mut *canvas, &*test_square, x as i64, y as i64)
    }

    canvas
}

#[test]
fn test_slow_merge() {
    let merged = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, 0, 0);
    assert_eq!(merged.width(), IMAGES_PER_ROW * 100);
    assert_eq!(merged.height(), IMAGES_PER_ROW * 100);
}

#[test]
fn test_slow_merge_padding() {
    let merged = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, PADDING_X, PADDING_Y);
    assert_eq!(
        merged.width(),
        IMAGES_PER_ROW * 100 + (PADDING_X * (IMAGES_PER_ROW - 1))
    );
    assert_eq!(
        merged.height(),
        TOTAL_ROWS * 100 + (PADDING_Y * (TOTAL_ROWS - 1))
    );
}

#[test]
fn test_push_merge() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(10, 100, 0, 0);

    let mut merger: FixedSizeMerger<Rgba<u8>> =
        FixedSizeMerger::new((100, 100), IMAGES_PER_ROW, TOTAL_ROWS, None);

    for _ in 0..100 {
        merger.push(&test_square);
    }

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_bulk_push_merge() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(10, 100, 0, 0);

    let mut merger: FixedSizeMerger<Rgba<u8>> =
        FixedSizeMerger::new((100, 100), IMAGES_PER_ROW, TOTAL_ROWS, None);
    merger.bulk_push(&vec![&test_square; 100]);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_push_merge_padding() {
    let test_square: Image<Rgba<u8>, RgbaImageBuffer<Vec<u8>>> = generate_test_square();
    let slow_merge = merge_images_slow(IMAGES_PER_ROW, TOTAL_IMAGES, PADDING_X, PADDING_Y);

    let mut merger: FixedSizeMerger<Rgba<u8>> = FixedSizeMerger::new(
        (100, 100),
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

    let mut merger: FixedSizeMerger<Rgba<u8>> = FixedSizeMerger::new(
        (100, 100),
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

    let mut merger: FixedSizeMerger<Rgba<u8>> =
        FixedSizeMerger::new((100, 100), IMAGES_PER_ROW, TOTAL_ROWS, None);

    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);
    merger.remove_image(99);

    assert_eq!(merger.get_canvas(), &slow_merge);
}
