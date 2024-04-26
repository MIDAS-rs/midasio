#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

/// The error type returned when parsing a MIDAS file fails.
#[derive(Debug)]
pub struct ParseError {
    offset: usize,
    inner: winnow::error::ContextError,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsing stopped at byte offset `{}`", self.offset)?;
        if self.inner.context().next().is_some() {
            write!(f, " ({})", self.inner)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner
            .cause()
            .map(|v| v as &(dyn std::error::Error + 'static))
    }
}

pub use file::FileView;

pub mod data_bank;
pub mod event;
pub mod file;
