# blorb-rs
A library for using blorb files in rust

## Description
This library is an implementation of the Blorb 2.0.4 specification. The specification can be found
at the following web address. A copy of the specification can also be found in the top level of
this repository.
* http://www.eblong.com/zarf/blorb/blorb.html

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
