fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(options) = ascii_game::app::Options::parse()? {
        ascii_game::app::run(options)?;
    }
    Ok(())
}
