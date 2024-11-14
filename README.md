# QOI (Quite OK Image Format) Rust De-/Encoder

This is a rust CLI application and library to decode and encode raw pixel data to and from the .qoi image format.
See [qoiformat.org](https://qoiformat.org/) for more information on the format. Both decoder and encoder pass all the test images provided by the format's maintainers.

I have created this largely to learn rust myself and do not recommend using this crate over the [Image crate](https://crates.io/crates/image).
Please observe that this crate is licensed under the GPL-v3-or-later only and can thus not be for non-FOSS projects. No MIT/BSD dual-licensing will be considered.

## To install

Simply grab the [latest release](https://git.valhrafnaz.gay/valhrafnaz/qoi-img/releases/latest) and place the binary in a path that is searched via your `$PATH` variable. Please keep in mind that the release binary provided is for linux-x86_64 only (code uses u64 and as thus is likely not functional on IA-32). Should you wish to run the program on different operating systems, please refer to build instructions below.

## To build

Make sure rustup has cargo and your preferred toolchain installed.

Clone the repository by running `git clone https://git.valhrafnaz.gay/valhrafnaz/qoi-img.git`

Move into the directory `cd qoi-img`

Run `cargo build -r` to build a stable version for your rustc toolchain in `./target/release`. 
