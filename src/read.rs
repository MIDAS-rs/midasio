//! # Quick Start
//!
//! To get you started quickly, the easiest and highest-level way to read a binary MIDAS file is
//! from a [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) slice. The [`read`] module
//! provides a useful API to inspect the data inside a MIDAS file.
//!
//! Reading directly from a [`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html) slice
//! allows us to, for example, use a memory-mapped file. See the
//! [`memmap2`](https://docs.rs/memmap2/latest/memmap2/) crate.

pub mod data_banks;
pub mod events;

#[cfg(test)]
mod tests;
