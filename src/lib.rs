#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub use file::FileView;

pub mod data_bank;
pub mod event;
pub mod file;
