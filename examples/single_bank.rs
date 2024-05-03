fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read("example.mid")?;
    // Note that if your MIDAS file is compressed, you will need to decompress
    // its contents (using an external crate) before parsing it.
    let file_view = midasio::FileView::try_from_bytes(&contents)?;

    // Iterate only through events with an ID of 1
    for event in file_view.into_iter().filter(|event| event.id() == 1) {
        // Lets assume that these events have multiple data banks, but they are
        // guaranteed to have a single bank with the name "TRGB" (which is the
        // one we are interested in)
        let [trg_bank] = event
            .into_iter()
            .filter(|bank| bank.name() == *b"TRGB")
            .collect::<Vec<_>>()[..]
        else {
            unreachable!();
        };
        // You can now access the data in the bank
        let _trg_data = trg_bank.data();
    }

    Ok(())
}
