# Midasio

[![Test Status](https://github.com/DJDuque/midasio/actions/workflows/rust.yml/badge.svg)](https://github.com/DJDuque/midasio/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/midasio?labelColor=383f47)](https://crates.io/crates/midasio)

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
