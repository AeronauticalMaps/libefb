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

//! AIXM 5.x parser.
//!
//! This crate reads AIXM 5.1, 5.1.1 and 5.2 documents as a stream and provides
//! the supported features as structures that follow the AIXM specification.
//!
//! [`read_message`] reads an `AIXMBasicMessage` into a [`Message`], which
//! keeps each feature with the BASELINE time slice that is valid on the
//! message's effective date. Features refer to each other by UUID, and the
//! message resolves a [`Reference`] on access. An invalid record doesn't
//! fail the message but is listed in [`Message::errors`].
//!
//! # Examples
//!
//! Lets read a message with one designated point and print its position:
//!
//! ```
//! # fn main() -> Result<(), aixm::Error> {
//! let xml = r#"
//!     <message:AIXMBasicMessage
//!         xmlns:message="http://www.aixm.aero/schema/5.1.1/message"
//!         xmlns:aixm="http://www.aixm.aero/schema/5.1.1"
//!         xmlns:gmd="http://www.isotc211.org/2005/gmd"
//!         xmlns:gml="http://www.opengis.net/gml/3.2">
//!         <aixm:messageMetadata>
//!             <gmd:EX_TemporalExtent>
//!                 <gmd:extent>
//!                     <gml:TimePeriod>
//!                         <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
//!                     </gml:TimePeriod>
//!                 </gmd:extent>
//!             </gmd:EX_TemporalExtent>
//!         </aixm:messageMetadata>
//!         <message:hasMember>
//!             <aixm:DesignatedPoint>
//!                 <gml:identifier>2b5a33f8-3b9c-4f0c-9a51-2f6b1c3d4e5f</gml:identifier>
//!                 <aixm:timeSlice>
//!                     <aixm:DesignatedPointTimeSlice>
//!                         <gml:validTime>
//!                             <gml:TimePeriod>
//!                                 <gml:beginPosition>2026-10-29T00:00:00Z</gml:beginPosition>
//!                                 <gml:endPosition>2026-11-26T00:00:00Z</gml:endPosition>
//!                             </gml:TimePeriod>
//!                         </gml:validTime>
//!                         <aixm:interpretation>BASELINE</aixm:interpretation>
//!                         <aixm:designator>GISDI</aixm:designator>
//!                         <aixm:location>
//!                             <aixm:Point srsName="urn:ogc:def:crs:EPSG::4326">
//!                                 <gml:pos>51.27 8.81</gml:pos>
//!                             </aixm:Point>
//!                         </aixm:location>
//!                     </aixm:DesignatedPointTimeSlice>
//!                 </aixm:timeSlice>
//!             </aixm:DesignatedPoint>
//!         </message:hasMember>
//!     </message:AIXMBasicMessage>"#;
//!
//! let message = aixm::read_message(xml.as_bytes())?;
//!
//! for point in message.designated_points() {
//!     let designator = point.designator.as_deref().unwrap_or_default();
//!     if let Some(location) = point.location {
//!         println!("{designator} at {}, {}", location.latitude, location.longitude);
//!     }
//! }
//! // => "GISDI at 51.27, 8.81"
//! # Ok(())
//! # }
//! ```

mod error;
mod features;
mod message;
mod stream;
mod timeslice;

pub use error::Error;
pub use features::{
    AirportHeliport, DesignatedPoint, Measure, Navaid, NavaidComponent, NavaidEquipment, Note,
    Point, Reference, Runway, RunwayCentrelinePoint, RunwayDeclaredDistance, RunwayDirection, DME,
    NDB, TACAN, VOR,
};
pub use message::{read_message, Message, Version};
