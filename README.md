# Midasio

[![Test Status](https://github.com/DJDuque/midasio/actions/workflows/rust.yml/badge.svg)](https://github.com/DJDuque/midasio/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/midasio?labelColor=383f47)](https://crates.io/crates/midasio)
[![GitHub commits since latest release (by date)](https://img.shields.io/github/commits-since/DJDuque/midasio/latest?labelColor=383f47)](https://github.com/DJDuque/midasio/commits/main)

A Rust library for reading binary MIDAS files. Midasio provides a useful API to
iterate over events, iterate over data banks, and extract the raw data from the
data banks.

## Usage

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
midasio = "0.4"
```
Reading a MIDAS file is as simple as:
```rust
use std::fs;
use midasio::read::file::FileView;

let contents = fs::read("example.mid")?;
let file_view = FileView::try_from(&contents[..])?;

for event in file_view {
    // Do something with each event in the file.
    for bank in event {
        // Do something with each data bank in the event.
    }
}
```

## Want to contribute?

There are multiple ways to contribute:
- Install and test Midasio. If it doesn't work as expected please [open an
  issue](https://github.com/DJDuque/midasio/issues/new).
- Comment/propose a fix on some of the current [open 
issues](https://github.com/DJDuque/midasio/issues).
- Read through the [documentation](https://docs.rs/midasio). If there is 
  something confusing, or you have a suggestion for something that could be 
  improved, please let the maintainer(s) know.
- Help evaluate [open pull requests](https://github.com/DJDuque/midasio/pulls),
  by testing locally and reviewing what is proposed.
