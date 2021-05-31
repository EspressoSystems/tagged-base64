// Copyright © 2021 Translucence Research, Inc. All rights reserved.

//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use base64::{decode_config, encode_config};
use std::str;
use tagged_base64::*;
use wasm_bindgen_test::*;
use web_sys;

// Run tests like this
//    wasm-pack test --headless --firefox --chrome
// Probably --safari works, too, but I'm not on a Mac at the moment.
//
// Note: sometimes there's a delay after the test results are reported
// but before the test runner exits.
//
// Comment this out to run tests in Node.js.
//    wasm-pack test --node
wasm_bindgen_test_configure!(run_in_browser);

/// Performs a brief sanity check on the base64 crate. Inspired by the
/// example at
/// https://rust-lang-nursery.github.io/rust-cookbook/encoding/strings.html
///
/// Checks the following
/// - Round trip correctness for a simple string
/// - The base64 encoding of the empty string is the empty string.
#[wasm_bindgen_test]
fn test_base64_sanity() {
    let hello = b"hello rustaceans";
    let encoded = encode_config(hello, TB64_CONFIG);
    let decoded = decode_config(&encoded, TB64_CONFIG).unwrap();
    assert_eq!(&hello.to_vec(), &decoded);
    assert_eq!(
        str::from_utf8(hello).unwrap(),
        str::from_utf8(&decoded).unwrap()
    );

    assert_eq!(decode_config("", TB64_CONFIG).unwrap().len(), 0);
}

#[wasm_bindgen_test]
fn test_base64_wrappers() {
    let x = b"abc123XYZ456";
    let e = TaggedBase64::encode_raw(x);
    assert_eq!(
        TaggedBase64::decode_raw(&e).expect("base64 decode failed"),
        x
    );
}

#[wasm_bindgen_test]
fn test_is_safe_base64_tag() {
    assert!(TaggedBase64::is_safe_base64_tag(""));
    assert!(!TaggedBase64::is_safe_base64_tag("~"));
    assert!(!TaggedBase64::is_safe_base64_tag("T~"));
    assert!(!TaggedBase64::is_safe_base64_tag("T~a"));
}

/// Compares to vectors of u8 for equality.
fn is_equal(va: &[u8], vb: &[u8]) -> bool {
    va.len() == vb.len() && va.iter().zip(vb).all(|(a, b)| *a == *b)
}

/// Rust n00b paranoia. Does my vector equality predicate work?
#[wasm_bindgen_test]
fn test_is_equal() {
    assert!(is_equal(&[], &[]));
    assert!(!is_equal(&[1], &[2]));
    assert!(is_equal(&[1, 2, 4], &[1, 2, 4]));
    assert!(!is_equal(&[1, 2, 4], &[1, 2, 4, 42]));
}

#[wasm_bindgen_test]
fn test_display() {
    let tb64 = TaggedBase64::new("T", b"123").unwrap();
    let str: String = tb64.to_string();
    let parsed: TaggedBase64 = TaggedBase64::parse(&str).unwrap();
    assert_eq!(tb64, parsed);
}

/// Checks basic construction, printing, and parsing:
/// - Can construct from a tag string and a binary value
/// - Tag and value match the supplied values
/// - String representation can be generated
/// - Generated string can be parsed
/// - Accessors and parsed string match the supplied values
fn check_tb64(tag: &str, value: &[u8]) {
    let tb64 = JsTaggedBase64::new(tag, &value).unwrap();
    let str = format!("{}", &tb64);

    // web_sys::console::log_1(&format!("{}", &tb64).into());

    let parsed = JsTaggedBase64::parse(&str).unwrap();

    assert_eq!(&tb64, &parsed);

    // Do we get back the tag we supplied?
    assert_eq!(parsed.tag(), tag);

    // Do we get back the binary value we supplied?
    assert!(is_equal(&parsed.value(), &value));
}

#[wasm_bindgen_test]
fn test_tagged_base64_parse() {
    // The empty string is not a valid TaggedBase64.
    assert!(JsTaggedBase64::parse("").is_err());

    // The tag is alphanumeric with hyphen and underscore.
    // The value here is the base64 encoding of foobar, but
    // the encoding doesn't include the required checksum.
    assert!(JsTaggedBase64::parse("-_~Zm9vYmFy").is_err());

    // A null value is not allowed.
    let b64_null = encode_config("", TB64_CONFIG);
    let tagged = format!("a~{}", &b64_null);
    assert!(JsTaggedBase64::parse(&tagged).is_err());

    // The tag can be empty, but the value cannot because the value
    // includes the checksum.
    assert!(JsTaggedBase64::parse("~").is_err());

    //-check_tb64("", b"");
    check_tb64("mytag", b"mytag");

    // Only base64 characters are allowed in the tag. No restrictions on
    // the value because it will get base64 encoded.
    check_tb64(
        "abcdefghijklmnopqrstuvwxyz-ABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789",
        "~Yeah, we can have spaces and odd stuff—😀 here. ¯⧵_(ツ)_/¯".as_bytes(),
    );
    check_tb64(
        "",
        b"abcdefghijklmnopqrstuvwxyz-ABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789~",
    );

    // All the following have invalid characters in the tag.
    assert!(JsTaggedBase64::new("~", b"").is_err());
    assert!(JsTaggedBase64::new("a~", b"").is_err());
    assert!(JsTaggedBase64::new("~b", b"").is_err());
    assert!(JsTaggedBase64::new("c~d", b"").is_err());
    assert!(JsTaggedBase64::new("e~f~", b"").is_err());
    assert!(JsTaggedBase64::new("g~h~i", b"").is_err());
    assert!(JsTaggedBase64::new("Oh, no!", b"").is_err());
    assert!(JsTaggedBase64::new("Σ", b"").is_err());

    // Note, u128::MAX is 340282366920938463463374607431768211455
    check_tb64("PK", &u128::MAX.to_string().as_bytes());

    // Is ten copies of u128::MAX a big enough test?
    let z = u128::MAX;
    check_tb64(
        "many-bits",
        &format!("{}{}{}{}{}{}{}{}{}{}", z, z, z, z, z, z, z, z, z, z).as_bytes(),
    );

    check_tb64("TX", b"transaction identifier goes here");
    check_tb64("KEY", b"public key bits");

    // From https://tools.ietf.org/html/rfc4648
    check_tb64("Zg", b"f");
    check_tb64("Zm8", b"fo");
    check_tb64("Zm9v", b"foo");
    check_tb64("Zm9vYg", b"foob");
    check_tb64("Zm9vYmE", b"fooba");
    check_tb64("Zm9vYmFy", b"foobar");
}

#[wasm_bindgen_test]
fn test_tagged_base64_new() {
    let bv = u128::MAX.to_ne_bytes().to_vec();
    let tb = TaggedBase64::new("BIG", &bv);
    assert!(is_equal(&tb.unwrap().value(), &bv));
}

#[wasm_bindgen_test]
fn test_tag_accessor() {
    let tag = "Tag47";
    let bits = b"Just some bits";
    let tb64 = TaggedBase64::new(&tag, bits).unwrap();
    assert_eq!(tb64.tag(), tag);
    assert_eq!(tb64.value(), bits);

    let jstb64 = JsTaggedBase64::new(&tag, bits).unwrap();
    assert_eq!(jstb64.tag(), tag);
    assert_eq!(jstb64.value(), bits);
}

#[wasm_bindgen_test]
fn test_tag_setter() {
    let tag = "Godzilla";
    let bits = b"forest";
    let mut tb64 = TaggedBase64::new("Bambi", bits).unwrap();
    tb64.set_tag(tag);
    assert_eq!(tb64.tag(), tag);
    assert_eq!(tb64.value(), bits);
}
