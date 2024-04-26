# Midasio

[![Test Status](https://github.com/MIDAS-rs/midasio/actions/workflows/rust.yml/badge.svg)](https://github.com/MIDAS-rs/midasio/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/midasio?labelColor=383f47)](https://crates.io/crates/midasio)

A Rust library for reading binary MIDAS files.

Midasio provides utilities to iterate over the MIDAS events in a file, iterate
over the data banks in a MIDAS event, and extract the raw data from the banks.

## Quick Start

To get you started quickly, the easiest and highest-level way to read a binary
MIDAS file is from a
[`&[u8]`](https://doc.rust-lang.org/std/primitive.slice.html). Parsing and
iterating over the contents of a file is as simple as:

```rust no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read("example.mid")?;
    let file_view = midasio::FileView::try_from_bytes(&contents[..])?;

    for event_view in file_view {
        // Do something with each event in the file.
        for bank_view in event_view {
            // Do something with each data bank in the event.
        }
    }

    Ok(())
}
```

## Feature flags

- `rayon`: Implement [`rayon`](https://crates.io/crates/rayon)'s 
`IntoParallelIterator` for `FileView`. This feature makes parallel analysis of
MIDAS events very easy with the `FileView::par_iter` and
`FileView::into_par_iter` methods.
