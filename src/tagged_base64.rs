// Copyright © 2021 Translucence Research, Inc. All rights reserved.
//
// See README.md for description and rationale.

use base64;
use core::fmt::Display;
use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq)]
pub struct TaggedBase64 {
    tag: String,
    value: Vec<u8>,
}

/// Separator that does not appear in URL-safe base64 encoding and can
/// appear in URLs without percent-encoding.
pub const TB64_DELIM: char = '~';

// Uses '-' and '_' as the 63rd and 64th characters. Does not use padding.
pub const TB64_CONFIG: base64::Config = base64::URL_SAFE_NO_PAD;

#[wasm_bindgen]
pub fn to_string(tb64: &TaggedBase64) -> String {
    format!(
        "{}{}{}",
        tb64.tag,
        TB64_DELIM,
        base64::encode_config(&tb64.value, TB64_CONFIG)
    )
}

/// Produces a string by concatenating the tag and the base64 encoding
/// of the value, separated by a tilde (~).
impl fmt::Display for TaggedBase64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.tag,
            TB64_DELIM,
            base64::encode_config(&self.value, TB64_CONFIG)
        )
    }
}

/// Checks that an ASCII byte is safe for use in the tag of a
/// TaggedBase64. Because the tags are merely intended to be mnemonic,
/// there's no need to support a large and visually ambiguous
/// character set.
#[wasm_bindgen]
pub fn is_safe_base64_tag(tag: &str) -> bool {
    tag.bytes()
        .skip_while(|b| is_safe_base64_ascii(*b as char))
        .next()
        .is_none()
}

/// Returns true for characters permitted in URL-safe base-64 encoding,
/// and false otherwise.
#[wasm_bindgen]
pub fn is_safe_base64_ascii(c: char) -> bool {
    ('a'..='z').contains(&c)
        || ('A'..='Z').contains(&c)
        || ('0'..='9').contains(&c)
        || (c == '-')
        || (c == '_')
}

#[wasm_bindgen]
impl TaggedBase64 {
    /// Constructs a TaggedBase64 from a tag and array of bytes. The tag
    /// must be URL-safe (alphanumeric with hyphen and underscore). The
    /// byte values are unconstrained.
    #[wasm_bindgen(constructor)]
    pub fn new(tag: &str, value: &[u8]) -> TaggedBase64 {
        assert!(is_safe_base64_tag(tag));
        TaggedBase64 {
            tag: tag.to_string(),
            value: value.to_vec(),
        }
    }

    /// Gets the tag of a TaggedBase64 instance.
    #[wasm_bindgen(getter)]
    pub fn tag(&self) -> String {
        self.tag.clone()
    }

    /// Sets the tag of a TaggedBase64 instance.
    #[wasm_bindgen(setter)]
    pub fn set_tag(&mut self, tag: &str) {
        assert!(is_safe_base64_tag(tag));
        self.tag = tag.to_string();
    }

    /// Gets the value of a TaggedBase64 instance.
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Vec<u8> {
        self.value.clone()
    }

    /// Sets the value of a TaggedBase64 instance.
    #[wasm_bindgen(setter)]
    pub fn set_value(&mut self, value: &[u8]) {
        self.value = value.to_vec();
    }
}

/// Parses a string of the form tag~value into a TaggedBase64 value.
///
/// The tag is restricted to URL-safe base64 ASCII characters. The tag
/// may be empty. The delimiter is required.
///
/// The value is a base64-encoded string, using the URL-safe character
/// set, and no padding is used.
#[wasm_bindgen]
pub fn tagged_base64_from(tb64: &str) -> Result<TaggedBase64, JsValue> {
    // Would be convenient to use split_first() here. Alas, not stable yet.
    let delim_pos = tb64
        .find(TB64_DELIM)
        .ok_or(to_jsvalue("Missing delimiter parsing TaggedBase64"))?;
    let (tag, delim_b64) = tb64.split_at(delim_pos);

    if !is_safe_base64_tag(tag) {
        return Err(to_jsvalue(format!(
            "Only alphanumeric ASCII, underscore (_), and hyphen (-) are allowed in the tag ({})",
            tag
        )));
    }

    // Remove the delimiter.
    let mut iter = delim_b64.chars();
    iter.next();
    let value = iter.as_str();

    // Base64 decode the value.
    let bytes = base64::decode_config(value, TB64_CONFIG).map_err(to_jsvalue)?;

    Ok(TaggedBase64 {
        tag: tag.to_string(),
        value: bytes,
    })
}

/// Constructs a TaggedBase64 from a tag string and a base64-encoded
/// value.
///
/// The tag is restricted to URL-safe base64 ASCII characters. The tag
/// may be empty. The delimiter is required.  The value is a a
/// base64-encoded string, using the URL-safe character set, and no
/// padding is used.
#[wasm_bindgen]
pub fn make_tagged_base64(tag: &str, value: &str) -> Result<TaggedBase64, JsValue> {
    if !is_safe_base64_tag(tag) {
        return Err(to_jsvalue(format!(
            "Only alphanumeric ASCII, underscore (_), and hyphen (-) are allowed in the tag ({})",
            tag
        )));
    }
    Ok(TaggedBase64 {
        tag: tag.to_string(),
        value: base64::decode_config(value, base64::URL_SAFE_NO_PAD)
            .map_err(|err| to_jsvalue(err))?,
    })
}

pub fn to_jsvalue<D: Display>(d: D) -> JsValue {
    JsValue::from_str(&format!("{}", d))
}
