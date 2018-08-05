# Simple Image Converter (sic)
Converts a single image from one format to another _(plus some other operations)_.

The image conversion is actually done by the awesome [image](https://crates.io/crates/image) crate  :balloon:.
`sic` is a small command line frontend which supports a portion of the conversion operations supported by the __image__ crate.

It was initially created to try out another awesome Rust library:  [clap](https://crates.io/crates/clap) :tada:


# Install

With [cargo](https://crates.io/crates/sic) install: `cargo install --force sic`

Pre build binary: see [releases](https://github.com/foresterre/sic/releases)

From the source of this repo:
- Setup rust and cargo with (for example) [rustup](https://rustup.rs/), a `nightly` version is required.
- Clone this repo: `git clone https://github.com/foresterre/sic.git`
- Switch to this repo: `cd sic`
- Build a release: `cargo build --release`


# Usage

**Convert an image from one format to another, for example from PNG to JPG.**
* In general: `sic <input> <output>`
* Example: `sic input.png output.jpg`

**Covert an image from one format to another while not caring about the output file extension.**
* In general `sic --force-format "<format>" <input> <output>` (or  `sic --force-format "<format>" <input> <output>`)
* Example `sic --force-format png input.bmp output.jpg` _(Note: `output.jpg` will have the PNG format even though the extension is `jpg`.)_

_Note: supported forced formats currently are: bmp, gif, ico, jpg (80%), pnm (P6 only). Support for other versions of supported formats is planned._

**Apply image operations to an image.**
* In general: `sic --script "<operations>" <input> <output> `
* Example `sic input.png output.jpg --script "flip_horizontal; blur 10; resize 250 250"

_Note: `resize` applies a gaussian sampling filter on resizing. This is currently the only sampling filter available (i.e. not changeable, all resize operations will be done with the gaussian sampling filter)._

Image operations availability:

```
operation => operation name
syntax => command syntax
available (from version) => lists whether the operation is supported right now (and optionally from which version forward)
description => descriptive information about the operation


<uint> means any 32 bit unsigned integer is required as parameter input
```

|operation|syntax|available (from version)|description|
|---|---|---|---|
|blur               | `blur <uint>`           | Yes (unreleased) | Performs a Gaussian blur on the image ([more info](https://docs.rs/image/0.19.0/image/imageops/fn.blur.html)) |
|brighten           | TBD                     | No               | ... |
|hue rotate         | TBD                     | No               | ... |
|contrast           | TBD                     | No               | ... |
|crop               | TBD                     | No               | ... |
|filter3x3          | TBD                     | No               | ... |
|flip horizontal    | `flip_horizontal`       | Yes (unreleased) | Flips the image on the horizontal axis |
|flip vertical      | `flip_vertical`         | Yes (unreleased) | Flips the image on the horizontal axis |
|gray scale         | TBD                     | No               | ... |
|resize             | `resize <uint> <uint>`  | Yes (unreleased) | Resize the image using a Gaussian sampling filter ([more info](https://docs.rs/image/0.19.0/image/imageops/fn.resize.html), [filter](https://docs.rs/image/0.19.0/image/enum.FilterType.html#variant.Gaussian)) |
|rotate90           | TBD                     | No               | ... |
|rotate180          | TBD                     | No               | ... |
|rotate270          | TBD                     | No               | ... |
|unsharpen          | TBD                     | No               | ... |
