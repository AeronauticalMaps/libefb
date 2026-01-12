use std::fs;
use std::hint::black_box;
use std::str::FromStr;

use arinc424::{Airport, Runway, Waypoint};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

const AIRPORT: &'static str = "SUSAP KJFKK6AJFK     0     145YHN40382374W073464329W013000013         1800018000C    MNAR    JOHN F KENNEDY INTL           300671912";
const WAYPOINT: &'static str = "SEURPCEDDHED W1    ED0    V     N53341894E009404512                                 WGE           WHISKEY1                 122922407";
const RUNWAY: &'static str = "SUSAP KJFKK6GRW04L   0120790440 N40372318W073470505         -0028300012046057200IIHIQ1                                     305541709";

/// Benchmark individual record parsing
fn bench_records(c: &mut Criterion) {
    c.bench_function("airport", |b| {
        b.iter(|| Airport::from_str(black_box(AIRPORT)))
    });

    c.bench_function("waypoint", |b| {
        b.iter(|| Waypoint::from_str(black_box(WAYPOINT)))
    });

    c.bench_function("runway", |b| b.iter(|| Runway::from_str(black_box(RUNWAY))));
}

/// Benchmark to own a alphanumeric and numeric field
fn bench_to_owned(c: &mut Criterion) {
    c.bench_function("alphanumeric to String", |b| {
        b.iter(|| {
            let aprt = Airport::from_str(black_box(AIRPORT)).expect("airport should parse");
            let _: String = aprt.icao_code.to_string();
        })
    });

    c.bench_function("numeric to u32", |b| {
        b.iter(|| {
            let rwy = Runway::from_str(black_box(RUNWAY)).expect("runway should parse");
            let _: u32 = rwy.runway_length.into();
        })
    });
}

/// Benchmark parsing the 50MB FAA file
fn bench_faa_cifp(c: &mut Criterion) {
    // Load file once
    let data = fs::read_to_string("FAACIFP18").expect("FAACIFP18 should be readable");

    let mut group = c.benchmark_group("FAA CIFP");

    // Tell Criterion the throughput for MB/s measurement
    group.throughput(Throughput::Bytes(data.len() as u64));

    // Benchmark: Just iterate over records (baseline)
    group.bench_function("baseline", |b| {
        b.iter(|| {
            let count = data.lines().count();
            black_box(count)
        })
    });

    // Benchmark: Parse all airports
    group.bench_function("airports", |b| {
        b.iter(|| {
            let mut count = 0;
            for chunk in data.lines() {
                // Section 'P', Subsection 'A' = Airport
                if &chunk[4..5] == "P" && &chunk[5..6] == "A" {
                    if let Ok(_) = Airport::from_str(chunk) {
                        count += 1;
                    }
                }
            }
            black_box(count)
        })
    });

    // Benchmark: Parse all runways
    group.bench_function("runways", |b| {
        b.iter(|| {
            let mut count = 0;
            for chunk in data.lines() {
                // Section 'P', Subsection 'G' = Runway
                if &chunk[4..5] == "P" && &chunk[5..6] == "G" {
                    if let Ok(_) = Runway::from_str(chunk) {
                        count += 1;
                    }
                }
            }
            black_box(count)
        })
    });

    // Benchmark: Parse all waypoints
    group.bench_function("waypoints", |b| {
        b.iter(|| {
            let mut count = 0;
            for chunk in data.lines() {
                // Section 'E', Subsection 'A' = Waypoint
                if &chunk[4..5] == "E" && &chunk[5..6] == "A" {
                    if let Ok(_) = Waypoint::from_str(chunk) {
                        count += 1;
                    }
                }
            }
            black_box(count)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_records, bench_to_owned, bench_faa_cifp);
criterion_main!(benches);
