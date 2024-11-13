# QOI (Quite OK Image Format) Rust De-/Encoder

This is a (currently not bug free) implementation of the QOI image compression algorithm in Rust.

Currently supports de- and encoding from and to PNG.

Current state is highly buggy and strangely only functions with 256 x 256 images.

## To build

run `cargo build -r` to build a stable version for your rustc toolchain in `./target/release`
