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

#![no_std]
#![allow(clippy::unused_unit)]
#[cfg(feature = "ark-serialize")]
use ark_serialize::*;
use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, Engine, GeneralPurpose},
};
use core::fmt;
#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
use core::fmt::Display;
use core::str::FromStr;
use crc_any::CRC;
#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer, Error as DeError},
    ser::{Error as SerError, Serialize, Serializer},
};
use snafu::Snafu;

use ark_std::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
use wasm_bindgen::prelude::*;

/// Derive serdes for a type which serializes as a binary blob.
///
/// This macro can be used to easily derive friendly serde implementations for a binary type which
/// implements [CanonicalSerialize](ark_serialize::CanonicalSerialize) and
/// [CanonicalDeserialize](ark_serialize::CanonicalDeserialize). This is useful for cryptographic
/// primitives and other types which do not have a human-readable serialization, but which may be
/// embedded in structs with a human-readable serialization. The serde implementations derived by
/// this macro will serialize the type as bytes for binary encodings and as base 64 for human
/// readable encodings.
///
/// This macro takes at least one arguments:
/// * The first argument should be the tag, as a string literal or expression.
/// * By default, the derived implementation invokes `CanonicalSerialize` and `CanonicalDeserialize`
///   with `uncompressed` and `unchecked` flags.
/// * If `compressed` and/or `checked` flags are presented, the derived implementation will behave
///   accordingly.
///
/// Specifically, this macro does 4 things when applied to a type definition:
/// * It adds `#[derive(Serialize, Deserialize)]` to the type definition, along with serde
///   attributes to serialize using [TaggedBase64].
/// * It creates an implementation of [Tagged] for the type using the specified tag. This tag will
///   be used to identify base 64 strings which represent this type in human-readable encodings.
/// * It creates an implementation of `TryFrom<TaggedBase64>` for the type `T`, which is needed to
///   make the `serde(try_from)` attribute work.
/// * It creates implementations of [Display](ark_std::fmt::Display) and
///   [FromStr](ark_std::str::FromStr) using tagged base 64 as a display format. This allows tagged
///   blob types to be conveniently displayed and read to and from user interfaces in a manner
///   consistent with how they are serialized.
///
/// Usage example:
///
/// ```
/// use ark_serialize::*;
/// use tagged_base64_macros::tagged;
///
/// #[tagged("PRIM")]
/// #[derive(Clone, CanonicalSerialize, CanonicalDeserialize, /* any other derives */)]
/// pub struct CryptoPrim(
///     // This type can only be serialied as an opaque, binary blob using ark_serialize.
///     pub(crate) ark_bls12_381::Fr,
/// );
/// ```
///
/// The type `CryptoPrim` can now be serialized as binary:
/// ```
/// # use ark_serialize::*;
/// # use ark_std::UniformRand;
/// # use tagged_base64_macros::tagged;
/// # use rand_chacha::{ChaChaRng, rand_core::SeedableRng};
/// # #[tagged("PRIM", compressed)]
/// # #[derive(Clone, CanonicalSerialize, CanonicalDeserialize, /* any other derives */)]
/// # struct CryptoPrim(ark_bls12_381::Fr);
/// # let crypto_prim = CryptoPrim(ark_bls12_381::Fr::rand(&mut ChaChaRng::from_seed([42; 32])));
/// bincode::serialize(&crypto_prim).unwrap();
/// ```
/// or as base64:
/// ```
/// # use ark_serialize::*;
/// # use ark_std::UniformRand;
/// # use tagged_base64_macros::tagged;
/// # use rand_chacha::{ChaChaRng, rand_core::SeedableRng};
/// # #[tagged("PRIM", compressed, checked)]
/// # #[derive(Clone, CanonicalSerialize, CanonicalDeserialize, /* any other derives */)]
/// # struct CryptoPrim(ark_bls12_381::Fr);
/// # let crypto_prim = CryptoPrim(ark_bls12_381::Fr::rand(&mut ChaChaRng::from_seed([42; 32])));
/// serde_json::to_string(&crypto_prim).unwrap();
/// ```
/// which will produce a tagged base64 string like
/// "PRIM~8oaujwbov8h4eEq7HFpqW6mIXhVbtJGxLUgiKrGpMCoJ".
pub use tagged_base64_macros::tagged;

/// Separator that does not appear in URL-safe base64 encoding and can
/// appear in URLs without percent-encoding.
pub const TB64_DELIM: char = '~';

/// Base 64 engine configured for TaggedBase64.
pub const BASE64: GeneralPurpose = GeneralPurpose::new(&URL_SAFE, NO_PAD);

/// A structure holding a string tag, vector of bytes, and a checksum
/// covering the tag and the bytes.
#[cfg_attr(all(target_arch = "wasm32", feature = "wasm-bindgen"), wasm_bindgen)]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "ark-serialize",
    derive(CanonicalSerialize, CanonicalDeserialize)
)]
pub struct TaggedBase64 {
    tag: String,
    value: Vec<u8>,
    checksum: u8,
}

#[cfg(feature = "serde")]
impl Serialize for TaggedBase64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            // If we are serializing to a human-readable format, be nice and just display the
            // tagged base 64 as a string.
            Serialize::serialize(&self.to_string(), serializer)
        } else {
            // For binary formats, convert to bytes (using CanonicalSerialize) and write the bytes.
            let mut bytes = Vec::new();
            CanonicalSerialize::serialize_compressed(self, &mut bytes).map_err(S::Error::custom)?;
            Serialize::serialize(&bytes, serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'a> Deserialize<'a> for TaggedBase64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        if deserializer.is_human_readable() {
            // If we are deserializing a human-readable format, the serializer would have written
            // the tagged base 64 as a string, so deserialize a string and then parse it. We need to
            // explicitly deserialize as an owned `String` before parsing. If we just did
            // `Self::from_str(&Deserialize::deserialize(...)?)`, the type for deserialization would
            // be inferred as `str`, and serde would try to borrow from the input, since `str` is
            // not a `Sized` type. Not all inputs support borrowing. For instance, this makes it
            // impossible to deserialize from a `serde_json::Value`.
            let s: String = Deserialize::deserialize(deserializer)?;
            Self::from_str(&s).map_err(D::Error::custom)
        } else {
            // Otherwise, this is a binary format; deserialize bytes and then convert the bytes to
            // TaggedBase64 using CanonicalDeserialize.
            let bytes = <Vec<u8> as Deserialize>::deserialize(deserializer)?;
            CanonicalDeserialize::deserialize_compressed_unchecked(bytes.as_slice())
                .map_err(D::Error::custom)
        }
    }
}

/// JavaScript-compatible wrapper for TaggedBase64
///
/// The primary difference is that JsTaggedBase64 returns errors
/// of type JsValue.
#[cfg_attr(all(target_arch = "wasm32", feature = "wasm-bindgen"), wasm_bindgen)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsTaggedBase64 {
    tb64: TaggedBase64,
}

#[derive(Debug, Snafu)]
pub enum Tb64Error {
    /// An invalid character was found in the tag.
    InvalidTag,
    /// Missing delimiter.
    MissingDelimiter,
    /// Missing checksum in value.
    MissingChecksum,
    #[snafu(display("invalid base 64: {message}"))]
    Base64 { message: String },
    /// The checksum was truncated or did not match.
    InvalidChecksum,
    /// The data did not encode the expected type.
    InvalidData,
}

impl From<base64::DecodeError> for Tb64Error {
    fn from(err: base64::DecodeError) -> Self {
        Self::Base64 {
            message: err.to_string(),
        }
    }
}

/// Converts a TaggedBase64 value to a String.
#[cfg_attr(all(target_arch = "wasm32", feature = "wasm-bindgen"), wasm_bindgen)]
pub fn to_string(tb64: &TaggedBase64) -> String {
    let value = &mut tb64.value.clone();
    value.push(tb64.checksum);
    format!(
        "{}{}{}",
        tb64.tag,
        TB64_DELIM,
        TaggedBase64::encode_raw(value)
    )
}

impl From<&TaggedBase64> for String {
    fn from(tb64: &TaggedBase64) -> Self {
        to_string(tb64)
    }
}

/// Produces the string of a TaggedBase64 value by concatenating the
/// tag, a delimeter, and the base64 encoding of the value and
/// checksum.
impl fmt::Display for TaggedBase64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_string(self))
    }
}

impl FromStr for TaggedBase64 {
    type Err = Tb64Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
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
        (crc8.get_crc() as u8) ^ (value.len() as u8)
    }

    /// Returns true for characters permitted in URL-safe base64 encoding,
    /// and false otherwise.
    pub fn is_safe_base64_ascii(c: char) -> bool {
        c.is_ascii_alphanumeric() || (c == '-') || (c == '_')
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
    // pub fn encode_raw<T: ?Sized + AsRef<[u8]>>(input: &T) -> String;
    pub fn encode_raw(input: &[u8]) -> String {
        BASE64.encode(input)
    }
    /// Wraps the underlying base64 decoder.
    pub fn decode_raw(value: &str) -> Result<Vec<u8>, Tb64Error> {
        Ok(BASE64.decode(value)?)
    }
}

impl AsRef<[u8]> for TaggedBase64 {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

/// Converts any object that supports the Display trait to a JsValue for
/// passing to Javascript.
///
/// Note: Type parameters aren't supported by `wasm-pack` yet so this
/// can't be included in the TaggedBase64 type implementation.
#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
pub fn to_jsvalue<D: Display>(d: D) -> JsValue {
    JsValue::from_str(&format!("{}", d))
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
impl From<Tb64Error> for JsValue {
    fn from(error: Tb64Error) -> JsValue {
        to_jsvalue(format!("{}", error))
    }
}

#[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen"))]
#[wasm_bindgen]
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

    /// Formats the JsTaggedBase64 instance as a URL-safe string.
    //
    // Note: this method is included for WASM bindings, since the trait methods from Display don't
    // get compiled to WASM.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        self.tb64.to_string()
    }
}

/// Trait for types whose serialization is not human-readable.
///
/// Such types have a human-readable tag which is used to identify tagged base
/// 64 blobs representing a serialization of that type.
///
/// Rather than implement this trait manually, it is recommended to use the
/// [macro@tagged] macro to specify a tag for your type. That macro also
/// derives appropriate serde implementations for serializing as an opaque blob.
pub trait Tagged {
    fn tag() -> String;
}
