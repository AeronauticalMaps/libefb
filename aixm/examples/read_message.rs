use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or("usage: read_message <file>")?;
    let message = aixm::read_message(BufReader::new(File::open(path)?))?;

    for error in message.errors() {
        eprintln!("{error}");
    }

    println!(
        "{} effective {}: {} airports, {} designated points, {} errors",
        message.version(),
        message.effective_date(),
        message.airport_heliports().count(),
        message.designated_points().count(),
        message.errors().len()
    );

    Ok(())
}
