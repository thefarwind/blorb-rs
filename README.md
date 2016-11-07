# blorb-rs
A library for using blorb files in rust

## Description
This library is an implementation of the Blorb 2.0.4 specification. The specification can be found
at the following web address. A copy of the specification can also be found in the top level of
this repository.
* http://www.eblong.com/zarf/blorb/blorb.html

Blorbs are a resource file type used in Interactive Fiction (IF). They bundle together images,
text, sounds, and other resources, along with executable code, for IF interpreters to use.

This library gives access to blorb and blorb file types. It provides structures to examine the
contents of blorbs and handle them in a logical way. Additionally, it provides a lazy access
interface to the blorb contents, allowing interpreters to use blorbs without handrolling their
own way to access the file without dumping the entire contents into memory.

## Usage

There are three primary interfaces to the blorb filetype provided in this library.

1. Blorb Reader
    * Provides methods for taking a blorb file and loading them into memory in a structured format.
    * ***TODO***: This has yet to be implemented.
2. Blorb Writer
    * Provides methods for taking blorb structures and turning them into blorb files.
    * ***TODO***: This has yet to be implemented.
3. Blorb Cursor
    * Provides methods to access blorb resources in a lazy manner.

### Blorb Cursor
The blorb cursor is implemented by the `Blorb` Structure. to create a `Blorb`, use the
`Blorb<R: Read + Seek>::from_file(file: R)` function. The function constructs a `Blorb` object
which manages moved file.

The `Blorb` object, when built, does some basic validation of the blorb file, and loads the
resource index and other blorb metadata from the file. It provides a `Blorb::load_resource(usize)`
method which can then be used to lazily load a blorb resource, such as an image or sound.

If the blorb contains an executable resource, it will be returned from calling
`Blorb::load_resource(0)`. The returned value from calling this method is a variant of the `Chunk`
enum, allowing the handling of the loaded resource to be done with a `match`.

## Build Instructions
The Blorb crate can be build using stable rust 1.12.1 and later.

To build, run the following command:
* `cargo build`

Before any changes are merged in, the following checks should be made:
* run `cargo build` and verify completion without any warnings
* run `cargo test` and verify completion without any warnings or test failures
* run `cargo doc` and verify completion without any issues
* run `rustup run nightly cargo build` and verify completion without any warnings
* run `rustup run nightly cargo test` and verify completion without any warnings or test failures
* run `rustup run nightly cargo doc` and verify completion without any issues
* run `rustup run nightly cargo clippy` and verify completion without any warnings
