// Copyright (C) 2017 1aim GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Dial-sequence notation defined by RFC 3601.
//!
//! A dial sequence (the RFC's `phone-string`) is the literal series of events
//! a device performs to place a call: DTMF digits and tones, one-second
//! pauses (`p`), waits for dial tone (`w`), and purely visual separators
//! (`-`, `.`). This is distinct from an E.164 number: it can contain tones and
//! timing that an E.164 number cannot, so it has its own parser.
//!
//! ```text
//! phone-string = 1*( DTMF / pause / tonewait / written-sep )
//! DTMF         = DIGIT / "#" / "*" / "A" / "B" / "C" / "D"
//! pause        = "p"        ; one-second pause (case-insensitive)
//! tonewait     = "w"        ; wait for dial tone (case-insensitive)
//! written-sep  = "-" / "."  ; visual only, ignored
//! ```

use std::fmt;

/// A single event in a dial sequence.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Element {
    /// A decimal digit `0`-`9`.
    Digit(char),
    /// A DTMF tone: `#`, `*`, or `A`-`D`.
    Tone(char),
    /// A one-second pause (`p`).
    Pause,
    /// A wait for dial tone (`w`).
    Wait,
}

/// A parsed RFC 3601 dial sequence.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DialSequence {
    elements: Vec<Element>,
}

impl DialSequence {
    /// The ordered events of the sequence. Visual separators (`-`, `.`) are not
    /// retained, as RFC 3601 defines them to be ignored.
    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// The decimal digits of the sequence, in order, with tones, pauses and
    /// separators removed.
    pub fn digits(&self) -> String {
        self.elements
            .iter()
            .filter_map(|e| match e {
                Element::Digit(d) => Some(*d),
                _ => None,
            })
            .collect()
    }
}

/// Parse an RFC 3601 dial sequence (`phone-string`).
///
/// Returns `None` if the input is empty or contains a character that is not a
/// DTMF event, pause, wait, or visual separator. A leading `+` (the RFC's
/// `global-phone` marker) is not part of a `phone-string` and is rejected.
pub fn parse(value: &str) -> Option<DialSequence> {
    let mut elements = Vec::new();

    for c in value.chars() {
        match c {
            '0'..='9' => elements.push(Element::Digit(c)),
            '#' | '*' | 'A' | 'B' | 'C' | 'D' => elements.push(Element::Tone(c)),
            'p' | 'P' => elements.push(Element::Pause),
            'w' | 'W' => elements.push(Element::Wait),
            // written-sep: visual only, dropped.
            '-' | '.' => {}
            _ => return None,
        }
    }

    // `phone-string` requires at least one event; separators alone do not count.
    if elements.is_empty() {
        None
    } else {
        Some(DialSequence { elements })
    }
}

impl fmt::Display for DialSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in &self.elements {
            match e {
                Element::Digit(c) | Element::Tone(c) => write!(f, "{c}")?,
                Element::Pause => write!(f, "p")?,
                Element::Wait => write!(f, "w")?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Element::*;
    use super::*;

    #[test]
    fn plain_digits() {
        assert_eq!(
            parse("12345").unwrap().elements(),
            &[Digit('1'), Digit('2'), Digit('3'), Digit('4'), Digit('5')]
        );
    }

    #[test]
    fn separators_are_dropped() {
        let seq = parse("1-800-555.0100").unwrap();
        assert_eq!(seq.digits(), "18005550100");
        assert!(!seq.elements().contains(&Tone('-')));
    }

    #[test]
    fn pauses_and_waits() {
        assert_eq!(
            parse("123pPwW").unwrap().elements(),
            &[
                Digit('1'),
                Digit('2'),
                Digit('3'),
                Pause,
                Pause,
                Wait,
                Wait,
            ]
        );
    }

    #[test]
    fn dtmf_tones() {
        assert_eq!(
            parse("*67#ABCD").unwrap().elements(),
            &[
                Tone('*'),
                Digit('6'),
                Digit('7'),
                Tone('#'),
                Tone('A'),
                Tone('B'),
                Tone('C'),
                Tone('D'),
            ]
        );
    }

    #[test]
    fn digits_strips_tones_and_timing() {
        assert_eq!(parse("*67w18005550100").unwrap().digits(), "6718005550100");
    }

    #[test]
    fn rejects_empty_and_separator_only() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("--.."), None);
    }

    #[test]
    fn rejects_invalid_and_leading_plus() {
        assert_eq!(parse("123x456"), None);
        assert_eq!(parse("12 34"), None);
        // '+' (global-phone) is not part of a phone-string.
        assert_eq!(parse("+123"), None);
    }

    #[test]
    fn display_roundtrips_without_separators() {
        assert_eq!(parse("1-800p555w0100").unwrap().to_string(), "1800p555w0100");
    }
}
