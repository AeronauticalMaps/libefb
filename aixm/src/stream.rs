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

use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};

use crate::Error;

/// The XML reader of a document.
///
/// A reader that holds an element while it reads the element's children
/// needs a buffer of its own for its events. An element that is read up
/// to its end tag without keeping its events -- a text or a skipped
/// element -- shares the stream's scratch buffer instead.
pub(crate) struct Stream<R> {
    reader: NsReader<R>,
    scratch: Vec<u8>,
}

impl<R: BufRead> Stream<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader: NsReader::from_reader(reader),
            scratch: Vec::new(),
        }
    }

    /// Reads the next event into `buf`.
    pub(crate) fn read_event_into<'b>(&mut self, buf: &'b mut Vec<u8>) -> Result<Event<'b>, Error> {
        Ok(self.reader.read_event_into(buf)?)
    }

    /// Reads the next event into `buf` and resolves its namespace.
    pub(crate) fn read_resolved_event_into<'b>(
        &mut self,
        buf: &'b mut Vec<u8>,
    ) -> Result<(ResolveResult<'_>, Event<'b>), Error> {
        Ok(self.reader.read_resolved_event_into(buf)?)
    }

    /// Reads the text of `element` up to its end tag and returns what
    /// `parse` makes of the trimmed text. The start tag of `element` must
    /// be the last event read.
    ///
    /// `parse` runs after the end tag is read, so an
    /// [`Error::InvalidRecord`] it returns leaves the reader behind the
    /// element.
    pub(crate) fn read_text<T>(
        &mut self,
        element: &BytesStart,
        parse: impl FnOnce(&str) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.scratch.clear();
        let text = self
            .reader
            .read_text_into(element.name(), &mut self.scratch)?
            .into_inner();
        let text = unescape(&text).map_err(quick_xml::Error::from)?;
        parse(text.trim())
    }

    /// Like [`Stream::read_text`] but returns `None` without calling
    /// `parse` if the element has no text, e.g. because it is `xsi:nil`.
    pub(crate) fn read_optional_text<T>(
        &mut self,
        element: &BytesStart,
        parse: impl FnOnce(&str) -> Result<T, Error>,
    ) -> Result<Option<T>, Error> {
        self.read_text(element, |text| {
            if text.is_empty() {
                Ok(None)
            } else {
                parse(text).map(Some)
            }
        })
    }

    /// Skips `element` up to its end tag. The start tag of `element` must
    /// be the last event read.
    pub(crate) fn skip(&mut self, element: &BytesStart) -> Result<(), Error> {
        self.scratch.clear();
        self.reader.read_to_end_into(element.name(), &mut self.scratch)?;
        Ok(())
    }
}

/// Copies a text that is kept, for [`Stream::read_text`].
pub(crate) fn owned(text: &str) -> Result<String, Error> {
    Ok(text.to_string())
}

/// Returns the value of the attribute of `element` with the local name
/// `name`. The value borrows from `element` unless it has to be
/// normalized.
pub(crate) fn attribute<'e>(
    element: &'e BytesStart,
    name: &str,
) -> Result<Option<Cow<'e, str>>, Error> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(quick_xml::Error::from)?;
        if attribute.key.local_name().as_ref() == name {
            return Ok(Some(attribute.normalized_value(XmlVersion::default())?));
        }
    }

    Ok(None)
}
