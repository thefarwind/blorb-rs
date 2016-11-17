//! This library is an implementation of the Blorb 2.0.4 specification.
//! The specification can be found at the following web address:
//!
//! * http://www.eblong.com/zarf/blorb/blorb.html
//!
//! Blorbs are a resource file type used in *Interactive Fiction* (IF).
//! They bundle together images, text, sounds, and other resources,
//! along with executable code, for IF interpreters to use.
//!
//! This library gives access to the blorb file type and contents
//! through structures. Additionally, it provides a lazy access
//! interface to the blorb contents, allowing interpreters to use blorbs
//! without dumping the full file contents contents into memory.
//!
//! **NOTE**: This library is not production ready. The interface is
//! currently unstable, and only the lazy-loading portion of this
//! library has been implemented.

extern crate byteorder;

mod blorb;
mod io;

pub use blorb::*;
pub use io::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
