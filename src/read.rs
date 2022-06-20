//! # Quick Start
//!
//! To get you started quickly, the easiest and highest-level way to read a binary MIDAS file is
//! from a [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) slice. The [`read`] module
//! provides a useful API to inspect the data inside a MIDAS file.
//!
//! ```no_run
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::fs;
//! use midasio::read::file::FileView;
//!
//! let contents = fs::read("example.mid")?;
//! let file_view = FileView::try_from(&contents[..])?;
//!
//! for event in file_view {
//!     // Do something with each event in the file
//!     // ...
//!     for bank in event {
//!         // Do something with each data bank
//!         // ...
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Reading directly from a [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) slice
//! allows us to, for example, use a memory-mapped file. See the
//! [`memmap2`](https://docs.rs/memmap2/latest/memmap2/) crate.

pub mod data_bank;
pub mod event;
pub mod file;

#[cfg(test)]
mod tests;
