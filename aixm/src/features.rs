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

use std::borrow::Cow;
use std::io::BufRead;

use chrono::{DateTime, Utc};
use quick_xml::events::{BytesStart, Event};

use crate::codes::{code, Code, CodeNotePurposeType};
use crate::stream::{attribute, owned, Stream};
use crate::timeslice::{select_valid, TimeSliceMetadata};
use crate::Error;

mod airport_heliport;
mod designated_point;
mod navaid;
mod runway;

pub use airport_heliport::AirportHeliport;
pub use designated_point::DesignatedPoint;
pub use navaid::{Navaid, NavaidComponent, NavaidEquipment, DME, NDB, TACAN, VOR};
pub use runway::{Runway, RunwayCentrelinePoint, RunwayDeclaredDistance, RunwayDirection};

const EPSG_4326: &str = "urn:ogc:def:crs:EPSG::4326";

/// A position in EPSG:4326 (`gml:pos` of an `aixm:Point` or
/// `aixm:ElevatedPoint`).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point {
    /// The latitude in degrees.
    pub latitude: f64,
    /// The longitude in degrees.
    pub longitude: f64,
}

impl Point {
    /// Reads the property `element` with an `aixm:Point` or
    /// `aixm:ElevatedPoint` up to its end tag. Returns `None` if the point
    /// has no position.
    fn read_location<R: BufRead>(
        stream: &mut Stream<R>,
        element: &str,
    ) -> Result<Option<Self>, Error> {
        let mut buf = Vec::new();
        // a wrong or missing srsName is kept only for the error
        let mut srs_name: Result<(), Option<String>> = Err(None);
        let mut pos = None;

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(start) => match start.local_name().as_ref() {
                    "Point" | "ElevatedPoint" => {
                        srs_name = match attribute(&start, "srsName")? {
                            Some(name) if name == EPSG_4326 => Ok(()),
                            name => Err(name.map(Cow::into_owned)),
                        };
                    }
                    // an invalid position fails only after the element is read
                    "pos" => {
                        pos = stream.read_optional_text(&start, |text| Ok(Self::from_pos(text)))?
                    }
                    _ => {}
                },
                Event::End(end) if end.local_name().as_ref() == element => break,
                Event::Eof => return Err(Error::InvalidXml(format!("{element} is not closed"))),
                _ => {}
            }
        }

        let Some(pos) = pos else {
            return Ok(None);
        };

        if let Err(srs_name) = srs_name {
            return Err(Error::InvalidRecord(format!(
                "position should be in {EPSG_4326} but is in {srs_name:?}"
            )));
        }

        pos.map(Some)
    }

    /// Parses a `gml:pos` with latitude and longitude.
    fn from_pos(pos: &str) -> Result<Self, Error> {
        let mut values = pos.split_whitespace().map(str::parse::<f64>);

        match (values.next(), values.next(), values.next()) {
            (Some(Ok(latitude)), Some(Ok(longitude)), None) => Ok(Self {
                latitude,
                longitude,
            }),
            _ => Err(Error::InvalidRecord(format!("{pos:?} is not a position"))),
        }
    }
}

/// A value with its unit of measurement `U`, e.g. a `ValDistanceType`
/// with a [`UomDistanceType`](crate::UomDistanceType) or a
/// `ValFrequencyType` with a [`UomFrequencyType`](crate::UomFrequencyType).
#[derive(Clone, PartialEq, Debug)]
pub struct Measure<U> {
    /// The value in the unit of measurement.
    pub value: f64,
    /// The unit of measurement (`uom`), e.g. FT, M or MHZ.
    pub uom: U,
}

impl<U> Measure<U> {
    /// Reads the measure `element` up to its end tag. Returns `None` if the
    /// element has no value.
    fn read<R: BufRead>(
        stream: &mut Stream<R>,
        element: &BytesStart,
    ) -> Result<Option<Self>, Error>
    where
        U: Code,
    {
        let uom = attribute(element, "uom")?.map(|uom| U::from_code(&uom));

        let value = stream.read_optional_text(element, |text| {
            text.parse::<f64>()
                .map_err(|_| Error::InvalidRecord(format!("{text:?} is not a measure")))
        })?;

        let Some(value) = value else {
            return Ok(None);
        };

        let uom =
            uom.ok_or_else(|| Error::InvalidRecord(format!("measure {value} has no uom")))?;

        Ok(Some(Self { value, uom }))
    }
}

/// Parses an angle in degrees, e.g. a ValBearingType or a
/// ValMagneticVariationType, which have no unit of measurement.
fn degrees(text: &str) -> Result<f64, Error> {
    text.parse()
        .map_err(|_| Error::InvalidRecord(format!("{text:?} is not in degrees")))
}

/// An `xlink` reference to another feature.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Reference {
    /// The UUID of the referenced feature, from `xlink:href` without the
    /// `urn:uuid:` prefix.
    pub uuid: String,
    /// The `xlink:title`, which names the referenced feature.
    pub title: Option<String>,
}

impl Reference {
    /// Reads the `xlink` attributes of `element`.
    fn from_element(element: &BytesStart) -> Result<Self, Error> {
        let href = attribute(element, "href")?
            .ok_or_else(|| Error::InvalidRecord("reference has no xlink:href".to_string()))?;

        let uuid = href
            .strip_prefix("urn:uuid:")
            .ok_or_else(|| Error::InvalidRecord(format!("{href:?} is not a UUID reference")))?;

        Ok(Self {
            uuid: uuid.to_string(),
            title: attribute(element, "title")?.map(Cow::into_owned),
        })
    }
}

/// A note of a feature (UML class `Note`).
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Note {
    /// The `purpose`, e.g. REMARK.
    pub purpose: Option<CodeNotePurposeType>,
    /// The texts of the `translatedNote`s.
    pub translated_note: Vec<String>,
}

impl Note {
    /// Reads an `aixm:annotation` up to its end tag.
    fn read_annotation<R: BufRead>(stream: &mut Stream<R>) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut note = Self::default();

        loop {
            buf.clear();
            match stream.read_event_into(&mut buf)? {
                Event::Start(element) => match element.local_name().as_ref() {
                    "purpose" => note.purpose = stream.read_optional_text(&element, code)?,
                    "note" => note.translated_note.push(stream.read_text(&element, owned)?),
                    _ => {}
                },
                Event::End(element) if element.local_name().as_ref() == "annotation" => break,
                Event::Eof => {
                    return Err(Error::InvalidXml("annotation is not closed".to_string()))
                }
                _ => {}
            }
        }

        Ok(note)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Feature Reader
/////////////////////////////////////////////////////////////////////////////

/// The properties of a feature's time slice, e.g. of a
/// `DesignatedPointTimeSlice`.
pub(crate) trait TimeSlice: Default {
    /// The local name of the feature, e.g. "DesignatedPoint".
    const FEATURE: &'static str;

    /// Reads the property `element` up to its end tag. The start tag of
    /// `element` must be the last event read.
    fn read_property<R: BufRead>(
        &mut self,
        stream: &mut Stream<R>,
        element: &BytesStart,
    ) -> Result<(), Error>;

    /// Reads the empty property `element`, e.g. an xlink reference.
    fn read_empty_property(&mut self, _element: &BytesStart) -> Result<(), Error> {
        Ok(())
    }
}

/// Reads the feature `T` up to its end tag and returns its identifier with
/// the time slice that is valid on `date`, or `None` if no time slice is
/// valid. The start tag must be the last event read.
///
/// An [`Error::InvalidRecord`] is returned only after the whole element is
/// read, so the reader can continue with the next feature.
pub(crate) fn read_feature<T: TimeSlice, R: BufRead>(
    stream: &mut Stream<R>,
    date: DateTime<Utc>,
) -> Result<Option<(String, T)>, Error> {
    let mut buf = Vec::new();
    let mut identifier = None;
    let mut time_slices = Vec::new();
    let mut record_error = None;

    loop {
        buf.clear();
        match stream.read_event_into(&mut buf)? {
            Event::Start(element) => match element.local_name().as_ref() {
                "identifier" => identifier = stream.read_optional_text(&element, owned)?,
                "timeSlice" => {
                    if let Some(time_slice) =
                        keep_record_error(read_time_slice(stream), &mut record_error)?
                    {
                        time_slices.push(time_slice);
                    }
                }
                _ => stream.skip(&element)?,
            },
            Event::End(_) => break,
            Event::Eof => return Err(Error::InvalidXml(format!("{} is not closed", T::FEATURE))),
            _ => {}
        }
    }

    let identifier = identifier.ok_or_else(|| {
        Error::InvalidRecord(format!("{} has no gml:identifier", T::FEATURE))
    })?;

    if let Some(message) = record_error {
        return Err(Error::InvalidRecord(format!(
            "{} {identifier}: {message}",
            T::FEATURE
        )));
    }

    Ok(select_valid(time_slices, date).map(|time_slice| (identifier, time_slice)))
}

/// Reads an `aixm:timeSlice` up to its end tag.
fn read_time_slice<T: TimeSlice, R: BufRead>(
    stream: &mut Stream<R>,
) -> Result<(TimeSliceMetadata, T), Error> {
    let mut buf = Vec::new();
    let mut time_slice = None;
    let mut record_error = None;

    loop {
        buf.clear();
        match stream.read_event_into(&mut buf)? {
            Event::Start(element)
                if element.local_name().as_ref().strip_suffix("TimeSlice") == Some(T::FEATURE) =>
            {
                time_slice = keep_record_error(read_properties(stream), &mut record_error)?
            }
            Event::Start(element) => stream.skip(&element)?,
            Event::End(_) => break,
            Event::Eof => return Err(Error::InvalidXml("timeSlice is not closed".to_string())),
            _ => {}
        }
    }

    if let Some(message) = record_error {
        return Err(Error::InvalidRecord(message));
    }

    time_slice.ok_or_else(|| {
        Error::InvalidRecord(format!("timeSlice has no {}TimeSlice", T::FEATURE))
    })
}

/// Reads the properties of a feature's time slice up to its end tag.
fn read_properties<T: TimeSlice, R: BufRead>(
    stream: &mut Stream<R>,
) -> Result<(TimeSliceMetadata, T), Error> {
    let mut buf = Vec::new();
    let mut metadata = TimeSliceMetadata::default();
    let mut time_slice = T::default();
    let mut record_error = None;

    loop {
        buf.clear();
        match stream.read_event_into(&mut buf)? {
            Event::Start(element) => {
                let result = match metadata.read_property(stream, &element) {
                    Ok(false) => time_slice.read_property(stream, &element),
                    other => other.map(|_| ()),
                };
                keep_record_error(result, &mut record_error)?;
            }
            Event::Empty(element) => {
                let result = time_slice.read_empty_property(&element);
                keep_record_error(result, &mut record_error)?;
            }
            Event::End(_) => break,
            Event::Eof => {
                return Err(Error::InvalidXml(format!(
                    "{}TimeSlice is not closed",
                    T::FEATURE
                )))
            }
            _ => {}
        }
    }

    match record_error {
        Some(message) => Err(Error::InvalidRecord(message)),
        None => Ok((metadata, time_slice)),
    }
}

/// Stores the message of a record error in `record_error` unless one is
/// already stored, and passes on every other error. This lets a reader
/// continue up to the end of its record after an invalid value.
fn keep_record_error<T>(
    result: Result<T, Error>,
    record_error: &mut Option<String>,
) -> Result<Option<T>, Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(Error::InvalidRecord(message)) => {
            record_error.get_or_insert(message);
            Ok(None)
        }
        Err(e) => Err(e),
    }
}
