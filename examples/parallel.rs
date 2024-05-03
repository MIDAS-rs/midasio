// To analyze events in parallel we need:
// - Add `rayon` as a dependency in your `Cargo.toml` (cargo add rayon)
// - Add `midasio` as a dependency with the `rayon` feature enabled (cargo add midasio --features rayon)
// - Bring rayon's prelude into scope (use rayon::prelude::*)

#[cfg(feature = "rayon")]
use rayon::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read("example.mid")?;
    // Note that if your MIDAS file is compressed, you will need to decompress
    // its contents (using an external crate) before parsing it.
    let file_view = midasio::FileView::try_from_bytes(&contents)?;

    // We want to process all events with an ID of 1. This can be any complex
    // operation, e.g. reconstructing a vertex
    let mut results = Vec::new();

    #[cfg(feature = "rayon")]
    results.par_extend(
        file_view
            .into_par_iter()
            .filter(|event| event.id() == 1)
            .map(|event| {
                // This is a placeholder for a complex operation
                event.into_iter().count()
            }),
    );
    // The equivalent non-parallel version would be:
    #[cfg(not(feature = "rayon"))]
    results.extend(
        file_view
            .into_iter()
            .filter(|event| event.id() == 1)
            .map(|event| event.into_iter().count()),
    );

    Ok(())
}
