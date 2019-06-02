warped-sampling
========
[![travis-ci.com](https://travis-ci.com/Jasper-Bekkers/warped-sampling.svg?branch=master)](https://travis-ci.com/Jasper-Bekkers/warped-sampling)
[![Latest version](https://img.shields.io/crates/v/warped-sampling.svg)](https://crates.io/crates/warped-sampling)
[![Documentation](https://docs.rs/warped-sampling/badge.svg)](https://docs.rs/warped-sampling)
[![](https://tokei.rs/b1/github/Jasper-Bekkers/warped-sampling)](https://github.com/Jasper-Bekkers/warped-sampling)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Rust implementation of the Warped Sampling technique for mipmaps from http://graphics.ucsd.edu/~henrik/papers/wavelet_importance_sampling.pdf

- [Documentation](https://docs.rs/warped-sampling)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
warped-sampling = "0.1.0"
```

Run the visual example as `cargo run --example visual` to generate some sample points.

### Input image
![Input](readme/Input.png?raw=true "Input")

### Output image
![Output](readme/Output.png?raw=true "Output")

## License

Licensed under MIT license (http://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.

Contributions are always welcome; please look at the [issue tracker](https://github.com/Jasper-Bekkers/warped-sampling/issues) to see what known improvements are documented.
