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

use quick_xml::escape::unescape;
use quick_xml::events::BytesStart;
use quick_xml::{NsReader, XmlVersion};

use crate::Error;

/// Reads the text of `element` up to its end tag. The start tag of `element`
/// must be the last event read.
pub(crate) fn read_text<R: BufRead>(
    reader: &mut NsReader<R>,
    element: &BytesStart,
) -> Result<String, Error> {
    let mut buf = Vec::new();
    let text = reader.read_text_into(element.name(), &mut buf)?.into_inner();
    let text = unescape(&text).map_err(quick_xml::Error::from)?;
    Ok(text.trim().to_string())
}

/// Like [`read_text`] but returns `None` if the element has no text, e.g.
/// because it is `xsi:nil`.
pub(crate) fn read_optional_text<R: BufRead>(
    reader: &mut NsReader<R>,
    element: &BytesStart,
) -> Result<Option<String>, Error> {
    let text = read_text(reader, element)?;
    Ok((!text.is_empty()).then_some(text))
}

/// Skips `element` up to its end tag. The start tag of `element` must be the
/// last event read.
pub(crate) fn skip<R: BufRead>(reader: &mut NsReader<R>, element: &BytesStart) -> Result<(), Error> {
    let mut buf = Vec::new();
    reader.read_to_end_into(element.name(), &mut buf)?;
    Ok(())
}

/// Returns the value of the attribute of `element` with the local name `name`.
pub(crate) fn attribute(element: &BytesStart, name: &str) -> Result<Option<String>, Error> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(quick_xml::Error::from)?;
        if attribute.key.local_name().as_ref() == name {
            return Ok(Some(attribute.normalized_value(XmlVersion::default())?.into_owned()));
        }
    }

    Ok(None)
}
