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

use std::io::BufRead;

use chrono::{DateTime, Utc};
use quick_xml::events::{BytesStart, Event};
use quick_xml::NsReader;

use crate::stream::read_text;
use crate::Error;

/// Properties that every AIXM time slice has (`AIXMTimeSlice`).
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub(crate) struct TimeSliceMetadata {
    pub(crate) valid_time: Option<ValidTime>,
    pub(crate) is_baseline: bool,
    pub(crate) sequence_number: Option<u32>,
    pub(crate) correction_number: Option<u32>,
}

impl TimeSliceMetadata {
    /// Reads `element` if it is a time slice metadata property and returns
    /// `true`, otherwise returns `false` without reading anything. The start
    /// tag of `element` must be the last event read.
    pub(crate) fn read_property<R: BufRead>(
        &mut self,
        reader: &mut NsReader<R>,
        element: &BytesStart,
    ) -> Result<bool, Error> {
        match element.local_name().as_ref() {
            "validTime" => self.valid_time = Some(ValidTime::read(reader, "validTime")?),
            "interpretation" => self.is_baseline = read_text(reader, element)? == "BASELINE",
            "sequenceNumber" => self.sequence_number = Some(read_number(reader, element)?),
            "correctionNumber" => self.correction_number = Some(read_number(reader, element)?),
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn is_valid_baseline_on(&self, date: DateTime<Utc>) -> bool {
        self.is_baseline
            && self
                .valid_time
                .as_ref()
                .is_some_and(|valid_time| valid_time.contains(date))
    }
}

/// The `gml:TimePeriod` of a `gml:validTime` or of a temporal extent. The end
/// is exclusive and `None` if the period lasts until further notice.
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct ValidTime {
    begin: DateTime<Utc>,
    end: Option<DateTime<Utc>>,
}

impl ValidTime {
    /// Reads the `gml:TimePeriod` within the element with the local name
    /// `element` up to the element's end tag.
    pub(crate) fn read<R: BufRead>(reader: &mut NsReader<R>, element: &str) -> Result<Self, Error> {
        let mut buf = Vec::new();
        let mut begin = None;
        let mut end = None;

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(start) => match start.local_name().as_ref() {
                    "beginPosition" => begin = read_time_position(reader, &start)?,
                    "endPosition" => end = read_time_position(reader, &start)?,
                    _ => {}
                },
                Event::End(close) if close.local_name().as_ref() == element => break,
                Event::Eof => return Err(Error::InvalidXml(format!("{element} is not closed"))),
                _ => {}
            }
        }

        let begin = begin
            .ok_or_else(|| Error::InvalidRecord(format!("{element} has no beginPosition")))?;

        Ok(Self { begin, end })
    }

    pub(crate) fn begin(&self) -> DateTime<Utc> {
        self.begin
    }

    fn contains(&self, date: DateTime<Utc>) -> bool {
        self.begin <= date && self.end.is_none_or(|end| date < end)
    }
}

/// Picks out of a feature's time slices the BASELINE that is valid on `date`.
/// If several are valid, the one with the highest sequence number and then
/// correction number is picked.
pub(crate) fn select_valid<T>(
    time_slices: impl IntoIterator<Item = (TimeSliceMetadata, T)>,
    date: DateTime<Utc>,
) -> Option<T> {
    time_slices
        .into_iter()
        .filter(|(metadata, _)| metadata.is_valid_baseline_on(date))
        .max_by_key(|(metadata, _)| (metadata.sequence_number, metadata.correction_number))
        .map(|(_, time_slice)| time_slice)
}

fn read_number<R: BufRead>(reader: &mut NsReader<R>, element: &BytesStart) -> Result<u32, Error> {
    let text = read_text(reader, element)?;
    text.parse()
        .map_err(|_| Error::InvalidRecord(format!("{text:?} is not a number")))
}

fn read_time_position<R: BufRead>(
    reader: &mut NsReader<R>,
    element: &BytesStart,
) -> Result<Option<DateTime<Utc>>, Error> {
    let text = read_text(reader, element)?;

    if text.is_empty() {
        return Ok(None);
    }

    DateTime::parse_from_rfc3339(&text)
        .map(|time| Some(time.with_timezone(&Utc)))
        .map_err(|_| Error::InvalidRecord(format!("{text:?} is not a time position")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::skip;

    const DESIGNATED_POINT_TIME_SLICE: &str = r#"
        <aixm:DesignatedPointTimeSlice
            xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
            xmlns:gml="http://www.opengis.net/gml/3.2">
            <gml:validTime>
                <gml:TimePeriod>
                    <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
                    <gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00.000Z</gml:endPosition>
                </gml:TimePeriod>
            </gml:validTime>
            <aixm:interpretation>BASELINE</aixm:interpretation>
            <aixm:sequenceNumber>1</aixm:sequenceNumber>
            <aixm:correctionNumber>0</aixm:correctionNumber>
            <aixm:designator>GISDI</aixm:designator>
        </aixm:DesignatedPointTimeSlice>"#;

    fn read_metadata(xml: &str) -> Result<TimeSliceMetadata, Error> {
        let mut reader = NsReader::from_reader(xml.as_bytes());
        let mut buf = Vec::new();
        let mut metadata = TimeSliceMetadata::default();

        // enter the time slice element
        loop {
            buf.clear();
            if let Event::Start(_) = reader.read_event_into(&mut buf)? {
                break;
            }
        }

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(element) => {
                    if !metadata.read_property(&mut reader, &element)? {
                        skip(&mut reader, &element)?;
                    }
                }
                Event::End(_) | Event::Eof => return Ok(metadata),
                _ => {}
            }
        }
    }

    fn time(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .expect("test time should be RFC 3339")
            .with_timezone(&Utc)
    }

    fn baseline(begin: &str, end: Option<&str>, sequence: u32, correction: u32) -> TimeSliceMetadata {
        TimeSliceMetadata {
            valid_time: Some(ValidTime {
                begin: time(begin),
                end: end.map(time),
            }),
            is_baseline: true,
            sequence_number: Some(sequence),
            correction_number: Some(correction),
        }
    }

    #[test]
    fn reads_metadata_of_time_slice() {
        assert_eq!(
            read_metadata(DESIGNATED_POINT_TIME_SLICE),
            Ok(baseline(
                "2026-10-29T00:00:00Z",
                Some("2026-11-26T00:00:00Z"),
                1,
                0
            ))
        );
    }

    #[test]
    fn reads_end_position_without_value_as_open() {
        let xml = DESIGNATED_POINT_TIME_SLICE.replace(
            r#"<gml:endPosition indeterminatePosition="before">2026-11-26T00:00:00.000Z</gml:endPosition>"#,
            r#"<gml:endPosition indeterminatePosition="unknown"></gml:endPosition>"#,
        );
        assert_eq!(
            read_metadata(&xml).map(|metadata| metadata.valid_time),
            Ok(Some(ValidTime {
                begin: time("2026-10-29T00:00:00Z"),
                end: None,
            }))
        );
    }

    #[test]
    fn rejects_invalid_time_position() {
        let xml = DESIGNATED_POINT_TIME_SLICE.replace("2026-10-29T00:00:00Z", "29.10.2026");
        assert!(matches!(read_metadata(&xml), Err(Error::InvalidRecord(_))));
    }

    #[test]
    fn selects_baseline_valid_on_date() {
        let date = time("2026-10-29T00:00:00Z");
        let mut not_baseline = baseline("2026-10-01T00:00:00Z", None, 2, 0);
        not_baseline.is_baseline = false;

        assert_eq!(
            select_valid(
                [
                    (not_baseline, "not baseline"),
                    (baseline("2026-10-01T00:00:00Z", Some("2026-10-29T00:00:00Z"), 1, 0), "ended"),
                    (baseline("2026-10-29T00:00:00Z", Some("2026-11-26T00:00:00Z"), 1, 0), "valid"),
                    (baseline("2026-11-26T00:00:00Z", None, 1, 0), "future"),
                ],
                date
            ),
            Some("valid")
        );
    }

    #[test]
    fn selects_highest_sequence_then_correction_number() {
        let date = time("2026-10-29T00:00:00Z");
        let slice = |sequence, correction| baseline("2026-10-29T00:00:00Z", None, sequence, correction);

        assert_eq!(
            select_valid(
                [
                    (slice(1, 3), "1.3"),
                    (slice(2, 0), "2.0"),
                    (slice(2, 1), "2.1"),
                    (slice(1, 5), "1.5"),
                ],
                date
            ),
            Some("2.1")
        );
    }

    #[test]
    fn selects_nothing_if_no_time_slice_is_valid() {
        let date = time("2026-10-01T00:00:00Z");
        let slices = [(baseline("2026-10-29T00:00:00Z", None, 1, 0), "future")];
        assert_eq!(select_valid(slices, date), None);
    }
}
