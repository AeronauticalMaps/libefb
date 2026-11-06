# AIXM 5.x Parser

> [!NOTE]
> This library is incomplete and reads only a few features for now.

This crate reads AIXM 5.1, 5.1.1 and 5.2 messages as a stream, so a
national data set is never loaded into memory as a whole. Each
feature keeps the BASELINE time slice that is valid on the message's
effective date. Features refer to each other by UUID, and the message
resolves those references on access. An invalid record doesn't fail
the message but is listed in its errors.

The following example reads a message from a file and prints what it
contains:

```rust
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("ED_AirportHeliport.xml")?;
    let message = aixm::read_message(BufReader::new(file))?;

    for error in message.errors() {
        eprintln!("{error}");
    }

    println!(
        "{} effective {}: {} airports, {} designated points",
        message.version(),
        message.effective_date(),
        message.airport_heliports().count(),
        message.designated_points().count(),
    );

    Ok(())
}
```

The same runs as an example against any AIXM file:

```
cargo run -p aixm --example read_message -- <file>
```
