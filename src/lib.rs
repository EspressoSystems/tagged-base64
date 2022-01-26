// Copyright © 2022 Translucence Research, Inc. All rights reserved.

//! User-oriented format for binary data. Tagged Base64 is intended to
//! be used in user interfaces including URLs and text to be copied
//! and pasted without the need for additional encoding, such as
//! quoting or escape sequences. A checksum is included so that common
//! problems such as inadvertent deletions or typos can be caught
//! without knowing the structure of the binary data.
//!
//! To further reduce confusion, the values are prefixed with a tag
//! intended to disambiguate usage. Although not necessary for
//! correctness, developers and users may find it convenient to have a
//! usage hint enabling them to see at a glance whether something is a
//! transaction id or a ledger address, etc.
//!
//! For example,
//! ```text
//!    KEY~cHVibGljIGtleSBiaXRzBQ
//!    TX~dHJhbnNhY3Rpb24gaWRlbnRpZmllciBnb2VzIGhlcmUC
//!    Zg~Zgg
//!    mytag~bXl0YWd7
//! ```
//!
//! Like the base64 value, the tag is also restricted to the URL-safe
//! base64 character set.
//!
//! Note: It is allowed for the tag to be the empty string. The base64
//! portion cannot be empty; at a minimum, it will encode a single
//! byte checksum.
//!
//! The tag and delimiter help to avoid problems with binary values
//! that happen to parse as numbers. Large binary values don't fit
//! nicely into JavaScript numbers due to range and
//! representation. JavaScript numbers are represented as 64-bit
//! floating point numbers. This means that the largest unsigned
//! integer that can be represented is 2^53 - 1. Moreover, it is very
//! easy to accidentally coerce a string that looks like a number into
//! a JavaScript number, thus running the risk of loss of precision,
//! which is corruption.  Therefore, values are encoded in base64 to
//! allow safe transit to- and from JavaScript, including in URLs, as
//! well as display and input in a user interface.

use core::fmt;
use core::fmt::Display;
use crc_any::CRC;
use wasm_bindgen::prelude::*;

/// Separator that does not appear in URL-safe base64 encoding and can
/// appear in URLs without percent-encoding.
pub const TB64_DELIM: char = '~';

/// Uses '-' and '_' as the 63rd and 64th characters. Does not use padding.
pub const TB64_CONFIG: base64::Config = base64::URL_SAFE_NO_PAD;

/// A structure holding a string tag, vector of bytes, and a checksum
/// covering the tag and the bytes.
#[wasm_bindgen]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaggedBase64 {
    tag: String,
    value: Vec<u8>,
    checksum: u8,
}

/// JavaScript-compatible wrapper for TaggedBase64
///
/// The primary difference is that JsTaggedBase64 returns errors
/// of type JsValue.
#[wasm_bindgen]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsTaggedBase64 {
    tb64: TaggedBase64,
}

#[derive(Debug)]
pub enum Tb64Error {
    /// An invalid character was found in the tag.
    InvalidTag,
    /// Missing delimiter.
    MissingDelimiter,
    /// Missing checksum in value.
    MissingChecksum,
    /// An invalid byte was found while decoding the base64-encoded value.
    /// The offset and offending byte are provided.
    InvalidByte(usize, u8),
    /// The last non-padding input symbol's encoded 6 bits have
    /// nonzero bits that will be discarded. This is indicative of
    /// corrupted or truncated Base64. Unlike InvalidByte, which
    /// reports symbols that aren't in the alphabet, this error is for
    /// symbols that are in the alphabet but represent nonsensical
    /// encodings.
    InvalidLastSymbol(usize, u8),
    /// The length of the base64-encoded value is invalid.
    InvalidLength,
    /// The checksum was truncated or did not match.
    InvalidChecksum,
}

impl fmt::Display for Tb64Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tb64Error::InvalidTag =>
                write!(f, "An invalid character was found in the tag."),
            Tb64Error::MissingDelimiter =>
                write!(f, "Missing delimiter ({}).", TB64_DELIM),
            Tb64Error::MissingChecksum =>
                write!(f, "Missing checksum in value."),
            Tb64Error::InvalidByte(offset, byte) =>
                write!(f, "An invalid byte ({:#0x}) was found at offset {} while decoding the base64-encoded value. The offset and offending byte are provided.", byte, offset),
            Tb64Error::InvalidLastSymbol(offset, byte) => write!(f, "The last non-padding input symbol's encoded 6 bits have nonzero bits that will be discarded. This is indicative of corrupted or truncated Base64. Unlike InvalidByte, which reports symbols that aren't in the alphabet, this error is for symbols that are in the alphabet but represent nonsensical encodings. Invalid byte ({:#0x}) at offset {}.", byte, offset),
            Tb64Error::InvalidLength =>
                write!(f, "The length of the base64-encoded value is invalid."),
            Tb64Error::InvalidChecksum =>
                write!(f, "The checksum was truncated or did not match."),
        }
    }
}

/// Converts a TaggedBase64 value to a String.
#[wasm_bindgen]
#[allow(clippy::unused_unit)]
pub fn to_string(tb64: &TaggedBase64) -> String {
    let value = &mut tb64.value.clone();
    value.push(TaggedBase64::calc_checksum(&tb64.tag, &tb64.value));
    format!(
        "{}{}{}",
        tb64.tag,
        TB64_DELIM,
        TaggedBase64::encode_raw(value)
    )
}

impl From<&TaggedBase64> for String {
    fn from(tb64: &TaggedBase64) -> Self {
        let value = &mut tb64.value.clone();
        value.push(TaggedBase64::calc_checksum(&tb64.tag, &tb64.value));
        format!(
            "{}{}{}",
            tb64.tag,
            TB64_DELIM,
            TaggedBase64::encode_raw(value)
        )
    }
}

/// Produces the string of a TaggedBase64 value by concatenating the
/// tag, a delimeter, and the base64 encoding of the value and
/// checksum.
impl fmt::Display for TaggedBase64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = &mut self.value.clone();
        value.push(TaggedBase64::calc_checksum(&self.tag, &self.value));
        write!(
            f,
            "{}{}{}",
            self.tag,
            TB64_DELIM,
            TaggedBase64::encode_raw(value)
        )
    }
}

/// Produces the string of a TaggedBase64 value by concatenating the
/// tag, a delimeter, and the base64 encoding of the value and
/// checksum.
impl fmt::Display for JsTaggedBase64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tb64)
    }
}

impl PartialEq<TaggedBase64> for JsTaggedBase64 {
    fn eq(&self, other: &TaggedBase64) -> bool {
        self.tb64 == *other
    }
}

impl TaggedBase64 {
    /// Constructs a TaggedBase64 from a tag and array of bytes. The tag
    /// must be URL-safe (alphanumeric with hyphen and underscore). The
    /// byte values are unconstrained.
    ///    ```ignored
    ///    use TaggedBase64;
    ///    let tb64 = TaggedBase64::new("TAG-YOURE-IT", b"datadatadata");
    ///    ```
    pub fn new(tag: &str, value: &[u8]) -> Result<TaggedBase64, Tb64Error> {
        if TaggedBase64::is_safe_base64_tag(tag) {
            let cs = TaggedBase64::calc_checksum(tag, value);
            Ok(TaggedBase64 {
                tag: tag.to_string(),
                value: value.to_vec(),
                checksum: cs,
            })
        } else {
            Err(Tb64Error::InvalidTag)
        }
    }

    /// Parses a string of the form tag~value into a TaggedBase64 value.
    ///
    /// The tag is restricted to URL-safe base64 ASCII characters. The tag
    /// may be empty. The delimiter is required.
    ///
    /// The value is a base64-encoded string, using the URL-safe character
    /// set, and no padding is used.
    pub fn parse(tb64: &str) -> Result<TaggedBase64, Tb64Error> {
        // Would be convenient to use split_first() here. Alas, not stable yet.
        let delim_pos = tb64.find(TB64_DELIM).ok_or(Tb64Error::MissingDelimiter)?;
        let (tag, delim_b64) = tb64.split_at(delim_pos);

        if !TaggedBase64::is_safe_base64_tag(tag) {
            return Err(Tb64Error::InvalidTag);
        }

        // Remove the delimiter.
        let mut iter = delim_b64.chars();
        iter.next();
        let value = iter.as_str();
        if value.is_empty() {
            return Err(Tb64Error::MissingChecksum);
        }

        // Note: 'printf' debugging is possible like this:
        //    use web_sys;
        //    web_sys::console::log_1(&format!("+ {}", &tb64).into());

        // Base64 decode the value.
        let bytes = TaggedBase64::decode_raw(value)?;
        let penultimate = bytes.len() - 1;
        let cs = bytes[penultimate];
        if cs == TaggedBase64::calc_checksum(tag, &bytes[..penultimate]) {
            Ok(TaggedBase64 {
                tag: tag.to_string(),
                value: bytes[..penultimate].to_vec(),
                checksum: cs,
            })
        } else {
            Err(Tb64Error::InvalidChecksum)
        }
    }

    fn calc_checksum(tag: &str, value: &[u8]) -> u8 {
        let mut crc8 = CRC::crc8();
        crc8.digest(&tag);
        crc8.digest(&value);
        crc8.get_crc() as u8
    }

    /// Returns true for characters permitted in URL-safe base64 encoding,
    /// and false otherwise.
    pub fn is_safe_base64_ascii(c: char) -> bool {
        ('a'..='z').contains(&c)
            || ('A'..='Z').contains(&c)
            || ('0'..='9').contains(&c)
            || (c == '-')
            || (c == '_')
    }

    /// Checks that an ASCII byte is safe for use in the tag of a
    /// TaggedBase64. Because the tags are merely intended to be mnemonic,
    /// there's no need to support a large and visually ambiguous
    /// character set.
    pub fn is_safe_base64_tag(tag: &str) -> bool {
        tag.chars().all(TaggedBase64::is_safe_base64_ascii)
    }

    /// Gets the tag of a TaggedBase64 instance.
    pub fn tag(&self) -> String {
        self.tag.clone()
    }

    /// Sets the tag of a TaggedBase64 instance.
    pub fn set_tag(&mut self, tag: &str) {
        assert!(TaggedBase64::is_safe_base64_tag(tag));
        self.tag = tag.to_string();
        self.checksum = TaggedBase64::calc_checksum(&self.tag, &self.value);
    }

    /// Gets the value of a TaggedBase64 instance.
    pub fn value(&self) -> Vec<u8> {
        self.value.clone()
    }

    /// Sets the value of a TaggedBase64 instance.
    pub fn set_value(&mut self, value: &[u8]) {
        self.value = value.to_vec();
        self.checksum = TaggedBase64::calc_checksum(&self.tag, &self.value);
    }

    /// Wraps the underlying base64 encoder.
    // WASM doesn't support the most general type.
    //
    // pub fn encode_raw<T: ?Sized + AsRef<[u8]>>(input: &T) -> String {
    //     base64::encode_config(input, TB64_CONFIG)
    // }
    pub fn encode_raw(input: &[u8]) -> String {
        base64::encode_config(input, TB64_CONFIG)
    }
    /// Wraps the underlying base64 decoder.
    pub fn decode_raw(value: &str) -> Result<Vec<u8>, Tb64Error> {
        base64::decode_config(value, TB64_CONFIG).map_err(|err| match err {
            base64::DecodeError::InvalidByte(offset, byte) => Tb64Error::InvalidByte(offset, byte),
            base64::DecodeError::InvalidLength => Tb64Error::InvalidLength,
            base64::DecodeError::InvalidLastSymbol(offset, byte) => {
                Tb64Error::InvalidLastSymbol(offset, byte)
            }
        })
    }
}

/// Converts any object that supports the Display trait to a JsValue for
/// passing to Javascript.
///
/// Note: Type parameters aren't supported by `wasm-pack` yet so this
/// can't be included in the TaggedBase64 type implementation.
pub fn to_jsvalue<D: Display>(d: D) -> JsValue {
    JsValue::from_str(&format!("{}", d))
}

impl From<Tb64Error> for JsValue {
    fn from(error: Tb64Error) -> JsValue {
        to_jsvalue(format!("{}", error))
    }
}

#[wasm_bindgen]
#[allow(clippy::unused_unit)]
impl JsTaggedBase64 {
    #[wasm_bindgen(constructor)]
    pub fn new(tag: &str, value: &[u8]) -> Result<JsTaggedBase64, JsValue> {
        let result = TaggedBase64::new(tag, value);
        match result {
            Ok(tb) => Ok(JsTaggedBase64 { tb64: tb }),
            Err(err) => Err(to_jsvalue(err)),
        }
    }

    /// Parses a string of the form tag~value into a TaggedBase64 value.
    ///
    /// The tag is restricted to URL-safe base64 ASCII characters. The tag
    /// may be empty. The delimiter is required.
    ///
    /// The value is a base64-encoded string, using the URL-safe character
    /// set, and no padding is used.
    pub fn parse(tb64: &str) -> Result<TaggedBase64, JsValue> {
        let result = TaggedBase64::parse(tb64)?;
        Ok(result)
    }

    /// Gets the tag of a TaggedBase64 instance.
    pub fn tag(&self) -> String {
        TaggedBase64::tag(&self.tb64)
    }

    /// Gets the value of a TaggedBase64 instance.
    pub fn value(&self) -> Vec<u8> {
        TaggedBase64::value(&self.tb64)
    }

    /// Sets the tag of a JsTaggedBase64 instance.
    pub fn set_tag(&mut self, tag: &str) {
        self.tb64.set_tag(tag);
    }

    /// Sets the value of a JsTaggedBase64 instance.
    pub fn set_value(&mut self, value: &[u8]) {
        self.tb64.set_value(value);
    }
}
