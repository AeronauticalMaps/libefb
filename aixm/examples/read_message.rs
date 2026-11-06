// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Joe Pearson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Lists the features of an AIXM message, one line per feature:
//!
//! ```text
//! cargo run -p aixm --example read_message -- <file>
//! ```

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use aixm::{Measure, Message, NavaidEquipment, Point};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or("usage: read_message <file>")?;
    let message = aixm::read_message(BufReader::new(File::open(path)?))?;

    for error in message.errors() {
        eprintln!("{error}");
    }

    println!(
        "{} effective {}, {} errors",
        message.version(),
        message.effective_date(),
        message.errors().len()
    );

    print_aerodromes(&message);
    print_runways(&message);
    print_navaids(&message);
    print_designated_points(&message);

    Ok(())
}

fn print_aerodromes(message: &Message) {
    for (_, arpt) in message.airport_heliports() {
        let ident = arpt
            .location_indicator_icao
            .as_ref()
            .or(arpt.designator.as_ref());

        println!(
            "{} {} at {}, {}, var {}",
            text(ident),
            text(arpt.name.as_ref()),
            position(arpt.arp),
            measure(arpt.field_elevation.as_ref()),
            arpt.magnetic_variation
                .map_or("-".to_string(), |degrees| degrees.to_string())
        );
    }
}

fn print_runways(message: &Message) {
    for (_, direction) in message.runway_directions() {
        // the direction refers to its runway, which refers to its aerodrome
        let Some(rwy) = direction
            .used_runway
            .as_ref()
            .and_then(|reference| message.runway(reference))
        else {
            continue;
        };

        let aerodrome = rwy
            .associated_airport_heliport
            .as_ref()
            .and_then(|reference| reference.title.as_ref());

        println!(
            "{} {} at {}°, {} {}",
            text(aerodrome),
            text(direction.designator.as_ref()),
            direction
                .magnetic_bearing
                .map_or("-".to_string(), |bearing| bearing.to_string()),
            measure(rwy.nominal_length.as_ref()),
            text(rwy.composition.as_ref())
        );
    }
}

fn print_navaids(message: &Message) {
    for navaid in message.navaids() {
        // the navaid refers to the equipment it consists of
        let equipment: Vec<String> = navaid
            .navaid_equipment
            .iter()
            .filter_map(|component| message.navaid_equipment(&component.the_navaid_equipment))
            .map(|equipment| match equipment {
                NavaidEquipment::VOR(vor) => format!("VOR {}", measure(vor.frequency.as_ref())),
                NavaidEquipment::DME(dme) => format!("DME {}", text(dme.channel.as_ref())),
                NavaidEquipment::NDB(ndb) => format!("NDB {}", measure(ndb.frequency.as_ref())),
                NavaidEquipment::TACAN(tacan) => {
                    format!("TACAN {}", text(tacan.channel.as_ref()))
                }
            })
            .collect();

        println!(
            "{} {} {}: {}",
            text(navaid.designator.as_ref()),
            text(navaid.kind.as_ref()),
            text(navaid.name.as_ref()),
            equipment.join(", ")
        );
    }
}

fn print_designated_points(message: &Message) {
    for point in message.designated_points() {
        println!(
            "{} {} at {}",
            text(point.designator.as_ref()),
            text(point.kind.as_ref()),
            position(point.location)
        );
    }
}

fn text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

fn position(point: Option<Point>) -> String {
    point.map_or("-".to_string(), |point| {
        format!("{:.4}, {:.4}", point.latitude, point.longitude)
    })
}

fn measure(measure: Option<&Measure>) -> String {
    measure.map_or("-".to_string(), |measure| {
        format!("{} {}", measure.value, measure.uom)
    })
}
