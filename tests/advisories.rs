// Regression tests for the published RFC3966 parsing panics:
//
//   * GHSA-whhr-7f2w-qqj2 (2023): panic on parsing crafted RFC3966 inputs.
//   * GHSA-mjw4-jj88-v687 (2024): panic on parsing crafted RFC3966 inputs.
//
// Both were out-of-bounds / non-char-boundary slicing panics triggered by
// numbers with empty or multibyte components around the `phone-context`
// parameter. Parsing must always return a `Result`, never panic.

use phonenumber::{country, parse};

/// Inputs that previously panicked or that exercise the same slicing paths:
/// empty components, bare separators, multibyte UTF-8 around boundaries, and
/// astral-plane characters.
const CRAFTED: &[&str] = &[
    // Published proof-of-concept inputs.
    ".;phone-context=",
    "+dwPAA;phone-context=AA",
    // Empty / minimal components.
    ";phone-context=",
    "tel:;phone-context=+",
    "tel:+;phone-context=",
    "+;phone-context=",
    "tel:;phone-context=",
    // Multibyte UTF-8 that must not be sliced mid-character.
    "+;phone-context=é",
    "tel:é;phone-context=+é",
    "+1;phone-context=ééé",
    "+\u{0080};phone-context=",
    // Astral-plane (4-byte) characters.
    "tel:0;phone-context=+\u{10000}",
    "+\u{1F4A9};phone-context=\u{1F4A9}",
    // Full-width / replacement characters.
    "tel:+\u{FF1B}phone-context=",
    "+\u{FFFC};phone-context=\u{FFFC}",
];

#[test]
fn crafted_rfc3966_inputs_never_panic() {
    for input in CRAFTED {
        // The only requirement is that this returns rather than panics.
        let _ = parse(None, input);
        let _ = parse(Some(country::US), input);
    }
}

#[test]
fn advisory_pocs_are_errors() {
    assert!(parse(None, ".;phone-context=").is_err());
    assert!(parse(None, "+dwPAA;phone-context=AA").is_err());
}
