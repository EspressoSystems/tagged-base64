// Copyright © 2021 Translucence Research, Inc. All rights reserved.

//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

use base64::{decode_config, encode_config};
use std::str;

use tagged_base64::*;

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

/// Checks basic construction, printing, and parsing:
/// - Can construct a TaggedBase64 from a tag string and a base64 string
/// - Tag and value match the supplied values
/// - String representation can be generated
/// - Generated string can be parsed
/// - Accessors and parsed string match the supplied values
fn check_tb64(tag: &str, lit: &str) {
    let b64 = encode_config(lit, TB64_CONFIG);
    let tb64 = make_tagged_base64(tag, &b64).unwrap();
    let str = format!("{}{}{}", &tag, TB64_DELIM, &b64);
    let parsed = tagged_base64_from(&str).unwrap();

    assert_eq!(&tb64, &parsed);

    // Do we get back the tag we supplied?
    assert_eq!(parsed.tag(), tag);

    // Do we get back the base64 value we supplied?
    assert_eq!(encode_config(parsed.value(), TB64_CONFIG), b64);
    let bits = base64::decode_config(b64, TB64_CONFIG).unwrap();
    assert!(is_equal(&parsed.value(), &bits));
}

#[wasm_bindgen_test]
fn test_tagged_base64_from() {
    // The empty string is not a valid TaggedBase64.
    assert!(tagged_base64_from("").is_err());

    // The tag is alphanumeric with hyphen and underscore.
    // The value here is the base64 encoding of foobar.
    assert!(tagged_base64_from("-_~Zm9vYmFy").is_ok());

    // A null value is allowed.
    let b64_null = encode_config("", TB64_CONFIG);
    let tagged = format!("a~{}", &b64_null);
    let short = tagged_base64_from(&tagged).unwrap();
    assert_eq!(&short.tag(), "a");
    assert_eq!(short.value().len(), 0);

    let tagged2 = format!("abc~{}", encode_config("31415", TB64_CONFIG));
    assert_eq!(
        encode_config(tagged_base64_from(&tagged2).unwrap().value(), TB64_CONFIG),
        "MzE0MTU"
    );

    let encode3 = encode_config("foobar", TB64_CONFIG);
    let tagged3 = format!("abc~{}", encode3);
    assert_eq!(
        encode_config(tagged_base64_from(&tagged3).unwrap().value(), TB64_CONFIG),
        encode3 // "Zm9vYmFy"
    );

    // Both the tag and the value can be empty.
    // TODO Should we prohibit this? It might be useful as a placeholder
    // but inadvertent whitespace could create confusion.
    assert!(tagged_base64_from("~").is_ok());

    check_tb64("", "");
    check_tb64("mytag", "mytag");

    // Only base64 characters are allowed in the tag. No restrictions on
    // the value because it will get base64 encoded.
    check_tb64(
        "abcdefghijklmnopqrstuvwxyz-ABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789",
        "~Yeah, we can have spaces and odd stuff—😀 here. ¯⧵_(ツ)_/¯",
    );
    check_tb64(
        "",
        "abcdefghijklmnopqrstuvwxyz-ABCDEFGHIJKLMNOPQRSTUVWXYZ_0123456789~",
    );

    // All the following have invalid characters in the tag.
    assert!(make_tagged_base64("~", "").is_err());
    assert!(make_tagged_base64("a~", "").is_err());
    assert!(make_tagged_base64("~b", "").is_err());
    assert!(make_tagged_base64("c~d", "").is_err());
    assert!(make_tagged_base64("e~f~", "").is_err());
    assert!(make_tagged_base64("g~h~i", "").is_err());
    assert!(make_tagged_base64("Oh, no!", "").is_err());
    assert!(make_tagged_base64("Σ", "").is_err());

    // Note, u128::MAX is 340282366920938463463374607431768211455
    check_tb64("PK", &u128::MAX.to_string());

    // Is ten copies of u128::MAX a big enough test?
    let z = u128::MAX;
    check_tb64(
        "many-bits",
        &format!("{}{}{}{}{}{}{}{}{}{}", z, z, z, z, z, z, z, z, z, z),
    );

    // From https://tools.ietf.org/html/rfc4648
    check_tb64("Zg", "f");
    check_tb64("Zm8", "fo");
    check_tb64("Zm9v", "foo");
    check_tb64("Zm9vYg", "foob");
    check_tb64("Zm9vYmE", "fooba");
    check_tb64("Zm9vYmFy", "foobar");
}

#[wasm_bindgen_test]
fn test_tagged_base64_new() {
    let bv = u128::MAX.to_ne_bytes().to_vec();
    let tb = TaggedBase64::new("BIG", &bv);
    assert!(is_equal(&tb.value(), &bv));
}
