# image-merger
This crate provides fast functionality for merging many images. It is built on top of the image crate and works to boost performance by utilizing parallel processing and avoiding unnecessary costly operations.
### What does it mean to "merge" images?
A Merger takes many small images and paces them onto a larger canvas in a specific pattern/location. As of today, this library only has one type of Merger, a `KnownSizeMerger` which focuses on performance as its top priority, but more will be added soon. An example of an output from a `KnownSizeMerger` is below, this is the general output from [the crate's tests](tests/known_size_merging.rs).

<img src="https://github.com/NextChai/image-merger/assets/75498301/a70fc92f-e5a6-4834-8ab0-37363cb2d178" width="250" height="250">
<img src="https://github.com/NextChai/image-merger/assets/75498301/ecdf0a62-e805-45ac-a2fc-5b4464c20f80" width="250" height="250">
<img src="https://github.com/NextChai/image-merger/assets/75498301/84d47b2d-cb79-42dc-aa91-553ceff9b7f4" width="250" height="250">

## Install

Run the following Cargo command in your project directory:

```
cargo add image-merger
```
## Benchmarks
### 100x100px Fixed-Size Images
The disparity in merging 10,000 images of 100x100 pixels between the merger and a linear implementation is significant. As depicted below, the x-axis illustrates the number of images being merged, ranging from 1 to 10,000, while the y-axis indicates the duration in milliseconds it took to merge all the images. The linear implementation is shown in green and the image merger in pink.

<img src="https://github.com/NextChai/image-merger/assets/75498301/cc2dd6a8-6d6c-421d-89f2-7efa96119abc" width="1080" height="480">

Over the 10,000 image interval, the image merger yielded an average **3.646x speed increase** over the linear implementation. Effectively speaking, the combined average time, in ms, to generate a merged image with $n100^2$ pixels is $A(n)= 0.0082n$, where $n$ is the number of merged images. Moreover, the average time to paste a single given pixel, in ms, is $A\left(n\right)=\frac{0.0082n}{100^{2}}$, where $n$ is the number of merged images.
