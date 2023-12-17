# image-merger
This crate provides fast functionality for merging many images. It is built on top of the image crate and works to boost performance by utilizing parallel processing and avoiding unnecessary costly operations.


## Install
Run the following Cargo command in your project directory:
```
cargo add image-merger
```
## Benchmarks
### 100x100px Fixed-Size Images
The disparity in merging 10,000 images of 100x100 pixels between the merger and a linear implementation is significant. As depicted below, the x-axis illustrates the number of images being merged, ranging from 1 to 10,000, while the y-axis indicates the duration in milliseconds it took to merge all the images. The linear implementation is shown in green and the image merger in pink.

<img src="https://github.com/NextChai/image-merger/assets/75498301/cc2dd6a8-6d6c-421d-89f2-7efa96119abc" width="1080" height="480">

Over the 10,000 image interval, the image merger yielded an average **3.646x speed increase** over the linear implementation, working in roughly O(0.0082n) time, where n is the number of images to merge into a single image.
