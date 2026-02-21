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

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::Error;
use crate::features::*;

/// Streaming iterator over AIXM features in an XML document.
///
/// Yields one [`Feature`] at a time as it encounters supported AIXM feature
/// elements in the XML stream. Unsupported elements are silently skipped.
///
/// # Examples
///
/// ```
/// use aixm::Features;
///
/// let xml = br#"
///   <message:AIXMBasicMessage
///     xmlns:aixm="http://www.aixm.aero/schema/5.1"
///     xmlns:gml="http://www.opengis.net/gml/3.2"
///     xmlns:message="http://www.aixm.aero/schema/5.1/message"
///     xmlns:xlink="http://www.w3.org/1999/xlink">
///     <message:hasMember>
///       <aixm:DesignatedPoint gml:id="uuid.abc">
///         <gml:identifier codeSpace="urn:uuid:">abc</gml:identifier>
///         <aixm:timeSlice>
///           <aixm:DesignatedPointTimeSlice gml:id="DP1">
///             <aixm:interpretation>BASELINE</aixm:interpretation>
///             <aixm:designator>ABLAN</aixm:designator>
///             <aixm:name>ABLAN</aixm:name>
///             <aixm:location>
///               <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326">
///                 <gml:pos>52.0 10.0</gml:pos>
///               </aixm:ElevatedPoint>
///             </aixm:location>
///           </aixm:DesignatedPointTimeSlice>
///         </aixm:timeSlice>
///       </aixm:DesignatedPoint>
///     </message:hasMember>
///   </message:AIXMBasicMessage>"#;
///
/// let features: Vec<_> = Features::new(&xml[..])
///     .collect::<Result<_, _>>()
///     .unwrap();
///
/// assert_eq!(features.len(), 1);
/// ```
pub struct Features<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
}

impl<'a> Features<&'a [u8]> {
    /// Creates a new `Features` iterator from a byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        let mut reader = Reader::from_reader(data);
        reader.config_mut().trim_text(true);
        Self {
            reader,
            buf: Vec::new(),
        }
    }
}

impl<R: BufRead> Features<R> {
    /// Creates a new `Features` iterator from any buffered reader.
    pub fn from_reader(reader: R) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        Self {
            reader: xml_reader,
            buf: Vec::new(),
        }
    }
}

impl<R: BufRead> Iterator for Features<R> {
    type Item = Result<Feature, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    let result = match local {
                        b"AirportHeliport" => {
                            let uuid = extract_gml_id(e);
                            parse_airport_heliport(&mut self.reader, uuid)
                                .map(Feature::AirportHeliport)
                        }
                        b"Runway" => {
                            let uuid = extract_gml_id(e);
                            parse_runway(&mut self.reader, uuid).map(Feature::Runway)
                        }
                        b"RunwayDirection" => {
                            let uuid = extract_gml_id(e);
                            parse_runway_direction(&mut self.reader, uuid)
                                .map(Feature::RunwayDirection)
                        }
                        b"DesignatedPoint" => {
                            let uuid = extract_gml_id(e);
                            parse_designated_point(&mut self.reader, uuid)
                                .map(Feature::DesignatedPoint)
                        }
                        b"Navaid" => {
                            let uuid = extract_gml_id(e);
                            parse_navaid(&mut self.reader, uuid).map(Feature::Navaid)
                        }
                        b"Airspace" => {
                            let uuid = extract_gml_id(e);
                            parse_airspace(&mut self.reader, uuid).map(Feature::Airspace)
                        }
                        _ => continue,
                    };
                    return Some(result);
                }
                Ok(Event::Eof) => return None,
                Err(e) => return Some(Err(e.into())),
                _ => continue,
            }
        }
    }
}

/// Returns the local name of an XML element, stripping any namespace prefix.
fn local_name(name: &[u8]) -> &[u8] {
    name.iter()
        .position(|&b| b == b':')
        .map_or(name, |pos| &name[pos + 1..])
}

/// Extracts the `gml:id` attribute and strips the `uuid.` prefix if present.
fn extract_gml_id(e: &quick_xml::events::BytesStart<'_>) -> String {
    for attr in e.attributes().flatten() {
        let key = local_name(attr.key.as_ref());
        if key == b"id" {
            let val = String::from_utf8_lossy(&attr.value);
            return val.strip_prefix("uuid.").unwrap_or(&val).to_string();
        }
    }
    String::new()
}

/// Extracts an `xlink:href` attribute value, stripping `urn:uuid:` prefix.
fn extract_xlink_href(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = local_name(attr.key.as_ref());
        if key == b"href" {
            let val = String::from_utf8_lossy(&attr.value);
            return Some(val.strip_prefix("urn:uuid:").unwrap_or(&val).to_string());
        }
    }
    None
}

/// Extracts the `uom` attribute value from an element.
fn extract_uom(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in e.attributes().flatten() {
        if local_name(attr.key.as_ref()) == b"uom" {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

/// Reads text content until the end of the current element at the given depth.
fn read_element_text<R: BufRead>(reader: &mut Reader<R>) -> Result<String, Error> {
    let mut buf = Vec::new();
    let mut text = String::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Text(e) => {
                text.push_str(&e.unescape()?);
            }
            Event::End(_) => return Ok(text),
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

/// Skips the current element and all its children.
fn skip_element<R: BufRead>(reader: &mut Reader<R>) -> Result<(), Error> {
    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF while skipping".to_string())),
            _ => {}
        }
    }
}

/// Parses a `gml:pos` text content into (latitude, longitude).
fn parse_pos(text: &str) -> Result<(f64, f64), Error> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(Error::InvalidValue {
            field: "gml:pos",
            value: text.to_string(),
        });
    }
    let lat = parts[0].parse::<f64>().map_err(|_| Error::InvalidValue {
        field: "gml:pos latitude",
        value: parts[0].to_string(),
    })?;
    let lon = parts[1].parse::<f64>().map_err(|_| Error::InvalidValue {
        field: "gml:pos longitude",
        value: parts[1].to_string(),
    })?;
    Ok((lat, lon))
}

/// Parses a `gml:posList` text content into a list of (latitude, longitude) pairs.
fn parse_pos_list(text: &str) -> Result<Vec<(f64, f64)>, Error> {
    let values: Vec<f64> = text
        .split_whitespace()
        .map(|s| {
            s.parse::<f64>().map_err(|_| Error::InvalidValue {
                field: "gml:posList",
                value: s.to_string(),
            })
        })
        .collect::<Result<_, _>>()?;

    if !values.len().is_multiple_of(2) {
        return Err(Error::InvalidValue {
            field: "gml:posList",
            value: "odd number of coordinates".to_string(),
        });
    }

    Ok(values.chunks(2).map(|c| (c[0], c[1])).collect())
}

// ---------------------------------------------------------------------------
// Feature parsers
// ---------------------------------------------------------------------------

fn parse_airport_heliport<R: BufRead>(
    reader: &mut Reader<R>,
    uuid: String,
) -> Result<AirportHeliport, Error> {
    let mut arpt = AirportHeliport {
        uuid,
        designator: String::new(),
        name: String::new(),
        location_indicator_icao: None,
        iata_designator: None,
        field_elevation: None,
        field_elevation_uom: None,
        latitude: None,
        longitude: None,
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    let mut in_baseline = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"AirportHeliportTimeSlice" => {}
                    b"interpretation" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        in_baseline = text == "BASELINE";
                    }
                    b"designator" if in_baseline => {
                        arpt.designator = read_element_text(reader)?;
                        depth -= 1;
                    }
                    b"name" if in_baseline => {
                        arpt.name = read_element_text(reader)?;
                        depth -= 1;
                    }
                    b"locationIndicatorICAO" if in_baseline => {
                        arpt.location_indicator_icao = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"designatorIATA" if in_baseline => {
                        arpt.iata_designator = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"fieldElevation" if in_baseline => {
                        arpt.field_elevation_uom = extract_uom(e);
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        arpt.field_elevation = text.parse().ok();
                    }
                    b"ElevatedPoint" if in_baseline => {
                        // ARP point — look for gml:pos inside
                    }
                    b"pos" if in_baseline => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        if let Ok((lat, lon)) = parse_pos(&text) {
                            arpt.latitude = Some(lat);
                            arpt.longitude = Some(lon);
                        }
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(arpt);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

fn parse_runway<R: BufRead>(reader: &mut Reader<R>, uuid: String) -> Result<Runway, Error> {
    let mut rwy = Runway {
        uuid,
        designator: String::new(),
        nominal_length: None,
        length_uom: None,
        surface_composition: None,
        associated_airport_uuid: None,
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    let mut in_baseline = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"RunwayTimeSlice" => {}
                    b"interpretation" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        in_baseline = text == "BASELINE";
                    }
                    b"designator" if in_baseline => {
                        rwy.designator = read_element_text(reader)?;
                        depth -= 1;
                    }
                    b"nominalLength" if in_baseline => {
                        rwy.length_uom = extract_uom(e);
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        rwy.nominal_length = text.parse().ok();
                    }
                    b"composition" if in_baseline => {
                        rwy.surface_composition = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"associatedAirportHeliport" if in_baseline => {
                        rwy.associated_airport_uuid = extract_xlink_href(e);
                        skip_element(reader)?;
                        depth -= 1;
                    }
                    _ => {}
                }
            }
            Event::Empty(ref e) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"associatedAirportHeliport" && in_baseline {
                    rwy.associated_airport_uuid = extract_xlink_href(e);
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(rwy);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

fn parse_runway_direction<R: BufRead>(
    reader: &mut Reader<R>,
    uuid: String,
) -> Result<RunwayDirection, Error> {
    let mut rdn = RunwayDirection {
        uuid,
        designator: String::new(),
        true_bearing: None,
        magnetic_bearing: None,
        used_runway_uuid: None,
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    let mut in_baseline = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"RunwayDirectionTimeSlice" => {}
                    b"interpretation" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        in_baseline = text == "BASELINE";
                    }
                    b"designator" if in_baseline => {
                        rdn.designator = read_element_text(reader)?;
                        depth -= 1;
                    }
                    b"trueBearing" if in_baseline => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        rdn.true_bearing = text.parse().ok();
                    }
                    b"magneticBearing" if in_baseline => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        rdn.magnetic_bearing = text.parse().ok();
                    }
                    b"usedRunway" if in_baseline => {
                        rdn.used_runway_uuid = extract_xlink_href(e);
                        skip_element(reader)?;
                        depth -= 1;
                    }
                    _ => {}
                }
            }
            Event::Empty(ref e) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local == b"usedRunway" && in_baseline {
                    rdn.used_runway_uuid = extract_xlink_href(e);
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(rdn);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

fn parse_designated_point<R: BufRead>(
    reader: &mut Reader<R>,
    uuid: String,
) -> Result<DesignatedPoint, Error> {
    let mut dp = DesignatedPoint {
        uuid,
        designator: String::new(),
        name: None,
        point_type: None,
        latitude: None,
        longitude: None,
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    let mut in_baseline = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"DesignatedPointTimeSlice" => {}
                    b"interpretation" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        in_baseline = text == "BASELINE";
                    }
                    b"designator" if in_baseline => {
                        dp.designator = read_element_text(reader)?;
                        depth -= 1;
                    }
                    b"name" if in_baseline => {
                        dp.name = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"type" if in_baseline => {
                        dp.point_type = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"pos" if in_baseline => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        if let Ok((lat, lon)) = parse_pos(&text) {
                            dp.latitude = Some(lat);
                            dp.longitude = Some(lon);
                        }
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(dp);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

fn parse_navaid<R: BufRead>(reader: &mut Reader<R>, uuid: String) -> Result<Navaid, Error> {
    let mut nav = Navaid {
        uuid,
        designator: String::new(),
        name: None,
        navaid_type: None,
        latitude: None,
        longitude: None,
        elevation: None,
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    let mut in_baseline = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"NavaidTimeSlice" => {}
                    b"interpretation" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        in_baseline = text == "BASELINE";
                    }
                    b"type" if in_baseline => {
                        nav.navaid_type = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"designator" if in_baseline => {
                        nav.designator = read_element_text(reader)?;
                        depth -= 1;
                    }
                    b"name" if in_baseline => {
                        nav.name = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"elevation" if in_baseline => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        nav.elevation = text.parse().ok();
                    }
                    b"pos" if in_baseline => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        if let Ok((lat, lon)) = parse_pos(&text) {
                            nav.latitude = Some(lat);
                            nav.longitude = Some(lon);
                        }
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(nav);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

fn parse_airspace<R: BufRead>(reader: &mut Reader<R>, uuid: String) -> Result<Airspace, Error> {
    let mut arsp = Airspace {
        uuid,
        airspace_type: None,
        designator: None,
        name: None,
        volumes: Vec::new(),
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;
    let mut in_baseline = false;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"AirspaceTimeSlice" => {}
                    b"interpretation" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        in_baseline = text == "BASELINE";
                    }
                    b"type" if in_baseline => {
                        arsp.airspace_type = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"designator" if in_baseline => {
                        arsp.designator = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"name" if in_baseline => {
                        arsp.name = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"AirspaceVolume" if in_baseline => {
                        let vol = parse_airspace_volume(reader)?;
                        // parse_airspace_volume consumes up to and including
                        // the </AirspaceVolume> end tag
                        depth -= 1;
                        arsp.volumes.push(vol);
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(arsp);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

fn parse_airspace_volume<R: BufRead>(reader: &mut Reader<R>) -> Result<AirspaceVolume, Error> {
    let mut vol = AirspaceVolume {
        upper_limit: None,
        upper_limit_uom: None,
        upper_limit_ref: None,
        lower_limit: None,
        lower_limit_uom: None,
        lower_limit_ref: None,
        polygon: Vec::new(),
    };

    let mut buf = Vec::new();
    let mut depth: u32 = 1;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                depth += 1;
                let name = e.name();
                let local = local_name(name.as_ref());
                match local {
                    b"upperLimit" => {
                        vol.upper_limit_uom = extract_uom(e);
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        vol.upper_limit = Some(text);
                    }
                    b"upperLimitReference" => {
                        vol.upper_limit_ref = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"lowerLimit" => {
                        vol.lower_limit_uom = extract_uom(e);
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        vol.lower_limit = Some(text);
                    }
                    b"lowerLimitReference" => {
                        vol.lower_limit_ref = Some(read_element_text(reader)?);
                        depth -= 1;
                    }
                    b"pos" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        if let Ok((lat, lon)) = parse_pos(&text) {
                            vol.polygon.push((lat, lon));
                        }
                    }
                    b"posList" => {
                        let text = read_element_text(reader)?;
                        depth -= 1;
                        vol.polygon.extend(parse_pos_list(&text)?);
                    }
                    _ => {}
                }
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return Ok(vol);
                }
            }
            Event::Eof => return Err(Error::Xml("unexpected EOF".to_string())),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_airport_heliport_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:AirportHeliport gml:id="uuid.dd062d88-3e64-4a5d-bebd-89476db9ebea">
              <gml:identifier codeSpace="urn:uuid:">dd062d88-3e64-4a5d-bebd-89476db9ebea</gml:identifier>
              <aixm:timeSlice>
                <aixm:AirportHeliportTimeSlice gml:id="AHP_EADH">
                  <gml:validTime>
                    <gml:TimePeriod gml:id="vt1">
                      <gml:beginPosition>2009-01-01T00:00:00Z</gml:beginPosition>
                      <gml:endPosition indeterminatePosition="unknown"/>
                    </gml:TimePeriod>
                  </gml:validTime>
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:sequenceNumber>1</aixm:sequenceNumber>
                  <aixm:designator>EADH</aixm:designator>
                  <aixm:name>DONLON/DOWNTOWN HELIPORT</aixm:name>
                  <aixm:locationIndicatorICAO>EADH</aixm:locationIndicatorICAO>
                  <aixm:fieldElevation uom="M">18</aixm:fieldElevation>
                  <aixm:ARP>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
                      <gml:pos>52.288888888888884 -32.035</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:ARP>
                </aixm:AirportHeliportTimeSlice>
              </aixm:timeSlice>
            </aixm:AirportHeliport>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::AirportHeliport(ahp) => {
                assert_eq!(ahp.uuid, "dd062d88-3e64-4a5d-bebd-89476db9ebea");
                assert_eq!(ahp.designator, "EADH");
                assert_eq!(ahp.name, "DONLON/DOWNTOWN HELIPORT");
                assert_eq!(ahp.location_indicator_icao.as_deref(), Some("EADH"));
                assert_eq!(ahp.field_elevation, Some(18.0));
                assert_eq!(ahp.field_elevation_uom.as_deref(), Some("M"));
                assert!((ahp.latitude.unwrap() - 52.2889).abs() < 0.001);
                assert!((ahp.longitude.unwrap() - (-32.035)).abs() < 0.001);
            }
            _ => panic!("expected AirportHeliport"),
        }
    }

    #[test]
    fn parse_runway_and_direction() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:Runway gml:id="uuid.9e51668f-bf8a-4f5b-ba6e-27087972b9b8">
              <gml:identifier codeSpace="urn:uuid:">9e51668f-bf8a-4f5b-ba6e-27087972b9b8</gml:identifier>
              <aixm:timeSlice>
                <aixm:RunwayTimeSlice gml:id="RWY1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>09L/27R</aixm:designator>
                  <aixm:nominalLength uom="M">2800.0</aixm:nominalLength>
                  <aixm:surfaceProperties>
                    <aixm:SurfaceCharacteristics gml:id="SC1">
                      <aixm:composition>CONC</aixm:composition>
                    </aixm:SurfaceCharacteristics>
                  </aixm:surfaceProperties>
                  <aixm:associatedAirportHeliport xlink:href="urn:uuid:1b54b2d6-a5ff-4e57-94c2-f4047a381c64"/>
                </aixm:RunwayTimeSlice>
              </aixm:timeSlice>
            </aixm:Runway>
          </message:hasMember>
          <message:hasMember>
            <aixm:RunwayDirection gml:id="uuid.c8455a6b-9319-4bb7-b797-08e644342d64">
              <gml:identifier codeSpace="urn:uuid:">c8455a6b-9319-4bb7-b797-08e644342d64</gml:identifier>
              <aixm:timeSlice>
                <aixm:RunwayDirectionTimeSlice gml:id="RDN1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>09L</aixm:designator>
                  <aixm:trueBearing>85.23</aixm:trueBearing>
                  <aixm:usedRunway xlink:href="urn:uuid:9e51668f-bf8a-4f5b-ba6e-27087972b9b8"/>
                </aixm:RunwayDirectionTimeSlice>
              </aixm:timeSlice>
            </aixm:RunwayDirection>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 2);

        match &features[0] {
            Feature::Runway(rwy) => {
                assert_eq!(rwy.designator, "09L/27R");
                assert_eq!(rwy.nominal_length, Some(2800.0));
                assert_eq!(rwy.length_uom.as_deref(), Some("M"));
                assert_eq!(rwy.surface_composition.as_deref(), Some("CONC"));
                assert_eq!(
                    rwy.associated_airport_uuid.as_deref(),
                    Some("1b54b2d6-a5ff-4e57-94c2-f4047a381c64")
                );
            }
            _ => panic!("expected Runway"),
        }

        match &features[1] {
            Feature::RunwayDirection(rdn) => {
                assert_eq!(rdn.designator, "09L");
                assert_eq!(rdn.true_bearing, Some(85.23));
                assert_eq!(
                    rdn.used_runway_uuid.as_deref(),
                    Some("9e51668f-bf8a-4f5b-ba6e-27087972b9b8")
                );
            }
            _ => panic!("expected RunwayDirection"),
        }
    }

    #[test]
    fn parse_designated_point_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:DesignatedPoint gml:id="uuid.abc123">
              <gml:identifier codeSpace="urn:uuid:">abc123</gml:identifier>
              <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="DP1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>ABLAN</aixm:designator>
                  <aixm:name>ABLAN</aixm:name>
                  <aixm:type>ICAO</aixm:type>
                  <aixm:location>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326">
                      <gml:pos>52.123 10.456</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:DesignatedPointTimeSlice>
              </aixm:timeSlice>
            </aixm:DesignatedPoint>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::DesignatedPoint(dp) => {
                assert_eq!(dp.designator, "ABLAN");
                assert_eq!(dp.name.as_deref(), Some("ABLAN"));
                assert_eq!(dp.point_type.as_deref(), Some("ICAO"));
                assert!((dp.latitude.unwrap() - 52.123).abs() < 0.001);
                assert!((dp.longitude.unwrap() - 10.456).abs() < 0.001);
            }
            _ => panic!("expected DesignatedPoint"),
        }
    }

    #[test]
    fn parse_navaid_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message"
          xmlns:xlink="http://www.w3.org/1999/xlink">
          <message:hasMember>
            <aixm:Navaid gml:id="uuid.08a1bbd5-ea70-4fe3-836a-ea9686349495">
              <gml:identifier codeSpace="urn:uuid:">08a1bbd5-ea70-4fe3-836a-ea9686349495</gml:identifier>
              <aixm:timeSlice>
                <aixm:NavaidTimeSlice gml:id="NAV_BOR">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:type>VOR_DME</aixm:type>
                  <aixm:designator>BOR</aixm:designator>
                  <aixm:name>BOORSPIJK</aixm:name>
                  <aixm:location>
                    <aixm:ElevatedPoint srsName="urn:ogc:def:crs:EPSG::4326" gml:id="ep1">
                      <gml:pos>52.368389 -32.375222</gml:pos>
                      <aixm:elevation uom="M">30.0</aixm:elevation>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:NavaidTimeSlice>
              </aixm:timeSlice>
            </aixm:Navaid>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::Navaid(nav) => {
                assert_eq!(nav.designator, "BOR");
                assert_eq!(nav.name.as_deref(), Some("BOORSPIJK"));
                assert_eq!(nav.navaid_type.as_deref(), Some("VOR_DME"));
                assert!((nav.latitude.unwrap() - 52.368389).abs() < 0.0001);
                assert!((nav.longitude.unwrap() - (-32.375222)).abs() < 0.0001);
                assert_eq!(nav.elevation, Some(30.0));
            }
            _ => panic!("expected Navaid"),
        }
    }

    #[test]
    fn parse_airspace_feature() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:Airspace gml:id="uuid.4fd9f4be-8c65-43f6-b083-3ced9a4b2a7f">
              <gml:identifier codeSpace="urn:uuid:">4fd9f4be-8c65-43f6-b083-3ced9a4b2a7f</gml:identifier>
              <aixm:timeSlice>
                <aixm:AirspaceTimeSlice gml:id="ASE1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:type>CTR</aixm:type>
                  <aixm:designator>EADD CTR</aixm:designator>
                  <aixm:name>DONLON CTR</aixm:name>
                  <aixm:geometryComponent>
                    <aixm:AirspaceGeometryComponent gml:id="AGC1">
                      <aixm:theAirspaceVolume>
                        <aixm:AirspaceVolume gml:id="AV1">
                          <aixm:upperLimit uom="FL">195</aixm:upperLimit>
                          <aixm:upperLimitReference>MSL</aixm:upperLimitReference>
                          <aixm:lowerLimit>GND</aixm:lowerLimit>
                          <aixm:lowerLimitReference>SFC</aixm:lowerLimitReference>
                          <aixm:horizontalProjection>
                            <aixm:Surface srsName="urn:ogc:def:crs:EPSG::4326" gml:id="S1">
                              <gml:patches>
                                <gml:PolygonPatch>
                                  <gml:exterior>
                                    <gml:Ring>
                                      <gml:curveMember>
                                        <gml:Curve gml:id="C1">
                                          <gml:segments>
                                            <gml:GeodesicString>
                                              <gml:posList>52.0 -32.0 52.5 -32.0 52.5 -31.5 52.0 -31.5 52.0 -32.0</gml:posList>
                                            </gml:GeodesicString>
                                          </gml:segments>
                                        </gml:Curve>
                                      </gml:curveMember>
                                    </gml:Ring>
                                  </gml:exterior>
                                </gml:PolygonPatch>
                              </gml:patches>
                            </aixm:Surface>
                          </aixm:horizontalProjection>
                        </aixm:AirspaceVolume>
                      </aixm:theAirspaceVolume>
                    </aixm:AirspaceGeometryComponent>
                  </aixm:geometryComponent>
                </aixm:AirspaceTimeSlice>
              </aixm:timeSlice>
            </aixm:Airspace>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        assert_eq!(features.len(), 1);

        match &features[0] {
            Feature::Airspace(arsp) => {
                assert_eq!(arsp.airspace_type.as_deref(), Some("CTR"));
                assert_eq!(arsp.designator.as_deref(), Some("EADD CTR"));
                assert_eq!(arsp.name.as_deref(), Some("DONLON CTR"));
                assert_eq!(arsp.volumes.len(), 1);

                let vol = &arsp.volumes[0];
                assert_eq!(vol.upper_limit.as_deref(), Some("195"));
                assert_eq!(vol.upper_limit_uom.as_deref(), Some("FL"));
                assert_eq!(vol.upper_limit_ref.as_deref(), Some("MSL"));
                assert_eq!(vol.lower_limit.as_deref(), Some("GND"));
                assert_eq!(vol.lower_limit_ref.as_deref(), Some("SFC"));
                assert_eq!(vol.polygon.len(), 5);
                assert!((vol.polygon[0].0 - 52.0).abs() < 0.001);
                assert!((vol.polygon[0].1 - (-32.0)).abs() < 0.001);
            }
            _ => panic!("expected Airspace"),
        }
    }

    #[test]
    fn skips_unsupported_features() {
        let xml = br#"
        <message:AIXMBasicMessage
          xmlns:aixm="http://www.aixm.aero/schema/5.1"
          xmlns:gml="http://www.opengis.net/gml/3.2"
          xmlns:message="http://www.aixm.aero/schema/5.1/message">
          <message:hasMember>
            <aixm:OrganisationAuthority gml:id="uuid.org1">
              <aixm:timeSlice>
                <aixm:OrganisationAuthorityTimeSlice gml:id="OA1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:name>SOME ORG</aixm:name>
                </aixm:OrganisationAuthorityTimeSlice>
              </aixm:timeSlice>
            </aixm:OrganisationAuthority>
          </message:hasMember>
          <message:hasMember>
            <aixm:DesignatedPoint gml:id="uuid.dp1">
              <aixm:timeSlice>
                <aixm:DesignatedPointTimeSlice gml:id="DPT1">
                  <aixm:interpretation>BASELINE</aixm:interpretation>
                  <aixm:designator>ALPHA</aixm:designator>
                  <aixm:location>
                    <aixm:ElevatedPoint>
                      <gml:pos>50.0 8.0</gml:pos>
                    </aixm:ElevatedPoint>
                  </aixm:location>
                </aixm:DesignatedPointTimeSlice>
              </aixm:timeSlice>
            </aixm:DesignatedPoint>
          </message:hasMember>
        </message:AIXMBasicMessage>"#;

        let features: Vec<_> = Features::new(&xml[..]).collect::<Result<_, _>>().unwrap();
        // OrganisationAuthority should be skipped
        assert_eq!(features.len(), 1);
        assert!(matches!(&features[0], Feature::DesignatedPoint(_)));
    }
}
