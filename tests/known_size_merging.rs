use image::imageops::replace;
use image_merger::*;

static IMAGES_PER_ROW: u32 = 10;
static TOTAL_ROWS: u32 = 10;
static TOTAL_IMAGES: u32 = 100;
static PADDING_X: u32 = 10;
static PADDING_Y: u32 = 10;
static IMAGE_WIDTH: u32 = 100;
static IMAGE_HEIGHT: u32 = 100;

type RgbaImageBuffer = BufferedImage<Rgba<u8>>;

/// Generates a test image of any pixel type whose subpixels depend on the coordinate, the channel
/// and the image's index, so that no two images in a merge are alike. Giving every image its own
/// content is what lets a merge catch an image that landed in the wrong place; pushing one image
/// repeatedly cannot.
fn generate_test_image<P>(width: u32, height: u32, index: u32) -> BufferedImage<P>
where
    P: Pixel<Subpixel = u8>,
{
    let channel_count = <P as Pixel>::CHANNEL_COUNT as usize;
    let mut image: BufferedImage<P> = BufferedImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let mut subpixels = [0u8; 4];
            for (channel, subpixel) in subpixels.iter_mut().take(channel_count).enumerate() {
                *subpixel = (x + (y * 3) + (index * 7) + (channel as u32 * 29)) as u8;
            }

            image.put_pixel(x, y, *P::from_slice(&subpixels[..channel_count]));
        }
    }

    image
}

/// Convenience wrapper for the square RGBA image the original tests were written against.
fn generate_test_square() -> RgbaImageBuffer {
    generate_test_image::<Rgba<u8>>(IMAGE_WIDTH, IMAGE_HEIGHT, 0)
}

/// Merges the given images the slow way, using the image crate, so that a merger's canvas can be
/// compared against a known good result.
///
/// Note that this replaces rather than overlays. The merger's paste does not blend, so an overlay
/// would only agree with it while every image is fully opaque.
fn merge_images_slow<P>(
    images: &[&BufferedImage<P>],
    image_dimensions: (u32, u32),
    images_per_row: u32,
    padding_x: u32,
    padding_y: u32,
) -> BufferedImage<P>
where
    P: Pixel<Subpixel = u8> + 'static,
{
    let (image_width, image_height) = image_dimensions;
    let total_images = images.len() as u32;

    // Cieling division for total rows.
    let total_rows = (total_images + images_per_row - 1) / images_per_row;

    let mut canvas: BufferedImage<P> = BufferedImage::new(
        image_width * images_per_row + (padding_x * (images_per_row - 1)),
        image_height * total_rows + (padding_y * (total_rows - 1)),
    );

    for (index, image) in images.iter().copied().enumerate() {
        let global_x = index as u32 % images_per_row;
        let global_y = index as u32 / images_per_row;

        let x = (global_x * image_width) + (global_x * padding_x);
        let y = (global_y * image_height) + (global_y * padding_y);

        replace(&mut *canvas, &**image, x as i64, y as i64)
    }

    canvas
}

/// Merges a set of distinct images through both `push` and `bulk_push` and checks each canvas
/// against a slow merge of the same images.
fn assert_merge_matches_slow<P>(
    image_dimensions: (u32, u32),
    images_per_row: u32,
    total_images: u32,
    padding: Option<Padding>,
) where
    P: Pixel<Subpixel = u8> + Sync + Send + PartialEq + std::fmt::Debug + 'static,
{
    let images: Vec<BufferedImage<P>> = (0..total_images)
        .map(|index| generate_test_image::<P>(image_dimensions.0, image_dimensions.1, index))
        .collect();
    let references: Vec<&BufferedImage<P>> = images.iter().collect();

    let padding_x = padding.map(|p| p.x).unwrap_or(0);
    let padding_y = padding.map(|p| p.y).unwrap_or(0);
    let slow_merge = merge_images_slow(
        &references,
        image_dimensions,
        images_per_row,
        padding_x,
        padding_y,
    );

    let mut pushed: KnownSizeMerger<P, _> =
        KnownSizeMerger::new(image_dimensions, images_per_row, total_images, padding);
    for image in &references {
        pushed.push(image);
    }
    assert_eq!(
        pushed.get_canvas(),
        &slow_merge,
        "push built a different canvas for {image_dimensions:?}"
    );

    let mut bulk_pushed: KnownSizeMerger<P, _> =
        KnownSizeMerger::new(image_dimensions, images_per_row, total_images, padding);
    bulk_pushed.bulk_push(&references);
    assert_eq!(
        bulk_pushed.get_canvas(),
        &slow_merge,
        "bulk_push built a different canvas for {image_dimensions:?}"
    );
}

#[test]
fn test_slow_merge() {
    let test_square = generate_test_square();
    let merged = merge_images_slow(
        &vec![&test_square; TOTAL_IMAGES as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        0,
        0,
    );

    assert_eq!(merged.width(), IMAGES_PER_ROW * IMAGE_WIDTH);
    assert_eq!(merged.height(), IMAGES_PER_ROW * IMAGE_HEIGHT);
}

#[test]
fn test_slow_merge_padding() {
    let test_square = generate_test_square();
    let merged = merge_images_slow(
        &vec![&test_square; TOTAL_IMAGES as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        PADDING_X,
        PADDING_Y,
    );

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
    let slow_merge = merge_images_slow(
        &vec![&test_square; TOTAL_IMAGES as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        0,
        0,
    );

    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );

    for _ in 0..TOTAL_IMAGES {
        merger.push(&test_square);
    }

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_bulk_push_merge() {
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(
        &vec![&test_square; TOTAL_IMAGES as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        0,
        0,
    );

    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );
    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_push_merge_padding() {
    let test_square: RgbaImageBuffer = generate_test_square();
    let slow_merge = merge_images_slow(
        &vec![&test_square; TOTAL_IMAGES as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        PADDING_X,
        PADDING_Y,
    );

    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
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
    let slow_merge = merge_images_slow(
        &vec![&test_square; TOTAL_IMAGES as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        PADDING_X,
        PADDING_Y,
    );

    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        Some(Padding {
            x: PADDING_X,
            y: PADDING_Y,
        }),
    );
    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_merge_distinct_images() {
    // Every other merge test pushes one image over and over, which cannot tell a correctly placed
    // image from one that landed at the wrong index.
    assert_merge_matches_slow::<Rgba<u8>>(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );
}

#[test]
fn test_merge_distinct_images_padding() {
    assert_merge_matches_slow::<Rgba<u8>>(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        Some(Padding {
            x: PADDING_X,
            y: PADDING_Y,
        }),
    );
}

#[test]
fn test_merge_non_square_images() {
    // A square image hides a width and height that have been swapped, so both orientations are
    // checked here.
    assert_merge_matches_slow::<Rgba<u8>>((37, 91), 5, 20, None);
    assert_merge_matches_slow::<Rgba<u8>>((91, 37), 5, 20, None);
}

#[test]
fn test_merge_non_square_images_padding() {
    assert_merge_matches_slow::<Rgba<u8>>((37, 91), 5, 20, Some(Padding { x: 7, y: 3 }));
    assert_merge_matches_slow::<Rgba<u8>>((91, 37), 5, 20, Some(Padding { x: 3, y: 7 }));
}

#[test]
fn test_merge_three_channel_images() {
    // Four channels per pixel divide evenly into most strides, so a channel count that is assumed
    // rather than read only shows up on a pixel type like this one.
    assert_merge_matches_slow::<Rgb<u8>>((53, 29), 4, 16, None);
    assert_merge_matches_slow::<Rgb<u8>>((53, 29), 4, 16, Some(Padding { x: 11, y: 5 }));
}

#[test]
fn test_merge_single_channel_images() {
    assert_merge_matches_slow::<Luma<u8>>((61, 17), 3, 9, None);
    assert_merge_matches_slow::<Luma<u8>>((61, 17), 3, 9, Some(Padding { x: 2, y: 9 }));
}

#[test]
fn test_merge_two_channel_images() {
    assert_merge_matches_slow::<LumaA<u8>>((23, 47), 4, 12, None);
    assert_merge_matches_slow::<LumaA<u8>>((23, 47), 4, 12, Some(Padding { x: 5, y: 2 }));
}

#[test]
fn test_merge_thin_and_tiny_images() {
    // Rows wide enough that a single image is split across threads, images short enough that many
    // of them are not, and the smallest image there is.
    assert_merge_matches_slow::<Rgba<u8>>((1024, 8), 2, 4, None);
    assert_merge_matches_slow::<Rgba<u8>>((8, 1024), 2, 4, None);
    assert_merge_matches_slow::<Rgba<u8>>((1, 1), 4, 16, None);
}

#[test]
fn test_merge_images_above_the_parallel_threshold() {
    // Large enough that pasting one image is split across threads, which the smaller cases above
    // never exercise.
    assert_merge_matches_slow::<Rgba<u8>>((512, 512), 2, 4, Some(Padding { x: 3, y: 3 }));
}

#[test]
fn test_push_image_with_oversized_container() {
    // The container behind an image is allowed to hold more than the image needs, so pasting one
    // has to copy only the rows that belong to it and leave the rest of the canvas alone.
    let channel_count = <Rgba<u8> as Pixel>::CHANNEL_COUNT as usize;
    let (width, height) = (40, 25);

    let mut container = vec![0u8; ((width * height) as usize * channel_count) + 4096];
    for (index, subpixel) in container.iter_mut().enumerate() {
        *subpixel = ((index % 250) + 1) as u8;
    }

    let image: RgbaImageBuffer = Image::new_from_raw(width, height, container)
        .expect("a container larger than the image is accepted");

    let mut merger: KnownSizeMerger<Rgba<u8>, _> =
        KnownSizeMerger::new((width, height), 2, 4, None);
    merger.push(&image);

    // Nothing was pushed into the second row of the canvas, so it must still be blank rather than
    // holding the tail of the oversized container.
    let canvas = merger.get_canvas();
    for y in height..(height * 2) {
        for x in 0..width {
            assert_eq!(
                canvas.get_pixel(x, y).0,
                [0, 0, 0, 0],
                "the canvas was written past the end of the image at ({x}, {y})"
            );
        }
    }
}

#[test]
fn test_remove_image() {
    // 99 images on the slow merge should be equal to 100 images on the fast merge minus the 1 removed image.
    let test_square = generate_test_square();
    let slow_merge = merge_images_slow(
        &vec![&test_square; (TOTAL_IMAGES - 1) as usize],
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        0,
        0,
    );

    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );

    merger.bulk_push(&vec![&test_square; TOTAL_IMAGES as usize]);
    merger.remove_image(99);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_remove_image_padding() {
    // Removing an image has to land on the same place the image was pasted, which on a padded
    // canvas is not the image's index times its size.
    let removed_index = 42;

    let images: Vec<RgbaImageBuffer> = (0..TOTAL_IMAGES)
        .map(|index| generate_test_image::<Rgba<u8>>(IMAGE_WIDTH, IMAGE_HEIGHT, index))
        .collect();
    let references: Vec<&RgbaImageBuffer> = images.iter().collect();

    // The removed image leaves blank space behind, so the expected canvas holds a blank image at
    // that index and the original image everywhere else.
    let blank: RgbaImageBuffer = BufferedImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let mut expected_images = references.clone();
    expected_images[removed_index as usize] = &blank;

    let slow_merge = merge_images_slow(
        &expected_images,
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        PADDING_X,
        PADDING_Y,
    );

    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        Some(Padding {
            x: PADDING_X,
            y: PADDING_Y,
        }),
    );

    merger.bulk_push(&references);
    merger.remove_image(removed_index);

    assert_eq!(merger.get_canvas(), &slow_merge);
}

#[test]
fn test_raw_create_image() {
    type P = Rgb<u8>;
    let container: Vec<u8> =
        vec![0; (IMAGE_WIDTH * IMAGE_HEIGHT * <P as image::Pixel>::CHANNEL_COUNT as u32) as usize];

    let image: Option<BufferedImage<P>> = Image::new_from_raw(IMAGE_WIDTH, IMAGE_HEIGHT, container);
    assert!(image.is_some());

    let image = image.unwrap();
    assert_eq!(image.width(), IMAGE_WIDTH);
    assert_eq!(image.height(), IMAGE_HEIGHT);
}

#[test]
fn test_raw_known_size_merger_create() {
    type P = Rgb<u8>;
    let container = vec![
        0;
        (IMAGE_WIDTH * IMAGE_HEIGHT * <P as image::Pixel>::CHANNEL_COUNT as u32 * TOTAL_IMAGES)
            as usize
    ];

    let merger: Option<KnownSizeMerger<P, Vec<u8>>> = KnownSizeMerger::new_from_raw(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
        container,
    );
    assert!(merger.is_some());

    let merger = merger.unwrap();
    assert_eq!(merger.get_canvas().width(), IMAGE_WIDTH * IMAGES_PER_ROW);
    assert_eq!(merger.get_canvas().height(), IMAGE_HEIGHT * TOTAL_ROWS);
}

#[test]
fn test_resizable_known_size_merger() {
    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );

    // Create an image that is twice the size of the original image. Using a known
    // resize function to ensure the image is resized correctly.
    let test_square = image::imageops::resize(
        &generate_test_square().into_buffer(),
        IMAGE_WIDTH * 2,
        IMAGE_HEIGHT * 2,
        image::imageops::FilterType::Nearest,
    );

    // Push the image into the merger.
    merger.push_resized(&Image::from(test_square));

    assert!(merger.get_num_images() == 1);
}

#[test]
fn test_resizable_bulk_known_size_merger() {
    let mut merger: KnownSizeMerger<Rgba<u8>, _> = KnownSizeMerger::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        IMAGES_PER_ROW,
        TOTAL_IMAGES,
        None,
    );

    // Create an image that is twice the size of the original image.
    let test_square = image::imageops::resize(
        &generate_test_square().into_buffer(),
        IMAGE_WIDTH * 2,
        IMAGE_HEIGHT * 2,
        image::imageops::FilterType::Nearest,
    );

    // Push the image into the merger.
    merger.bulk_push_resized(&vec![&Image::from(test_square); TOTAL_IMAGES as usize]);
    assert!(merger.get_num_images() == TOTAL_IMAGES);
}
