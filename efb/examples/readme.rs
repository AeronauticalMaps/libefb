use std::fs::read_to_string;
use std::path::Path;

use efb::prelude::*;

fn main() -> Result<(), Error> {
    let mut fms = FMS::new();

    // Read a ARINC 424 file downloaded from https://www.openflightmaps.org
    let records = read_to_string(Path::new("arinc_ed.pc")).unwrap_or_default();

    // Read the German navigation data from the ARINC 424 record
    let ed = NavigationData::try_from_arinc424(&records)?;
    fms.modify_nd(|nd| nd.append(ed))?;

    // Decode a route from EDDH to EDHF with winds at 20 kt from 290° and
    // cruising speed of 107 kt and an altitude of 2500 ft.
    fms.decode("29020KT N0107 A0250 EDDH N2 N1 DCT EDHF".to_string())?;

    // Now we could define an aircraft and continue with our planning
    // but for now we'll just print the route
    println!("{}", fms.print(40)); // the line length is set to 40 character

    Ok(())
}
