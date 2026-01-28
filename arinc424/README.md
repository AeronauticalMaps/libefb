# ARINC 424 Navigation Data Parser

> [!NOTE]
> This library is incomplete and parses only a few records for now.

This crate provides an iterator to iterate over ARINC 424
records. Records can then by converted from the fixed 132 bytes.

The following example iterates over two ARINC 424 records and print
some information of the records:

```rust
use arinc424::records::{Airport, RecordKind, Records, Runway};
use arinc424::Error;

const DATA: &'static [u8] = br#"
SUSAP KJFKK6AJFK     0     145YHN40382374W073464329W013000013         1800018000C    MNAR    JOHN F KENNEDY INTL           300671912
SUSAP KJFKK6GRW04L   0120790440 N40372318W073470505         -0028300012046057200IIHIQ1                                     305541709
"#;

fn main() -> Result<(), Error> {
    // create an iterator for the records
    let mut iter = Records::new(DATA);

    // the first record in our data is JFK airport
    if let Some((RecordKind::Airport, bytes)) = iter.next() {
        let airport = Airport::try_from(bytes)?;
        println!("Airport {} ({})", airport.airport_name, airport.arpt_ident);
    }

    // the second record in our data is runway 31R of JFK airport
    if let Some((RecordKind::Runway, bytes)) = iter.next() {
        let runway = Runway::try_from(bytes)?;
        println!(
            "Runway {} of {} is {}ft long",
            runway.runway_id.designator()?,
            runway.arpt_ident,
            runway.runway_length.as_u32()?
        );
    }

    Ok(())
}
```

This will print

```
Airport JOHN F KENNEDY INTL (KJFK)
Runway 04L of KJFK is 12079ft long
```
