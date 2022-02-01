// Copyright © 2022 Translucence Research, Inc. All rights reserved.

use quickcheck_macros::quickcheck;

use base64::{decode_config, encode_config};
use std::str;
use tagged_base64::*;

#[cfg(target_arch = "wasm32")]
use {wasm_bindgen::JsValue, wasm_bindgen_test::*};

// Run WASM tests like this
//    wasm-pack test --headless --firefox --chrome
// Probably --safari works, too, but I'm not on a Mac at the moment.
//
// Note: sometimes there's a delay after the test results are reported
// but before the test runner exits.
//
// Comment this out to run tests in Node.js.
//    wasm-pack test --node
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

/// Performs a brief sanity check on the base64 crate. Inspired by the
/// example at
/// https://rust-lang-nursery.github.io/rust-cookbook/encoding/strings.html
///
/// Checks the following
/// - Round trip correctness for a simple string
/// - The base64 encoding of the empty string is the empty string.
fn base64_sanity() {
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_base64_sanity() {
    base64_sanity();
}

#[test]
fn test_base64_sanity() {
    base64_sanity();
}

fn base64_wrappers() {
    let x = b"abc123XYZ456";
    let e = TaggedBase64::encode_raw(x);
    assert_eq!(
        TaggedBase64::decode_raw(&e).expect("base64 decode failed"),
        x
    );
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_base64_wrappers() {
    base64_wrappers();
}

#[test]
fn test_base64_wrappers() {
    base64_wrappers();
}

fn is_safe_base64_tag() {
    assert!(TaggedBase64::is_safe_base64_tag(""));
    assert!(!TaggedBase64::is_safe_base64_tag("~"));
    assert!(!TaggedBase64::is_safe_base64_tag("T~"));
    assert!(!TaggedBase64::is_safe_base64_tag("T~a"));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_is_safe_base64_tag() {
    is_safe_base64_tag();
}

#[test]
fn test_is_safe_base64_tag() {
    is_safe_base64_tag();
}

/// Compares to vectors of u8 for equality.
fn is_equal(va: &[u8], vb: &[u8]) -> bool {
    va.len() == vb.len() && va.iter().zip(vb).all(|(a, b)| *a == *b)
}

/// Rust n00b paranoia. Does my vector equality predicate work?
fn is_equal_tester() {
    assert!(is_equal(&[], &[]));
    assert!(!is_equal(&[1], &[2]));
    assert!(is_equal(&[1, 2, 4], &[1, 2, 4]));
    assert!(!is_equal(&[1, 2, 4], &[1, 2, 4, 42]));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_is_equal() {
    is_equal_tester();
}

#[test]
fn test_is_equal() {
    is_equal_tester();
}

fn display() {
    let tb64 = TaggedBase64::new("T", b"123").unwrap();
    let str: String = tb64.to_string();
    let parsed: TaggedBase64 = TaggedBase64::parse(&str).unwrap();
    assert_eq!(tb64, parsed);
    let from_tb64 = String::from(&tb64);
    assert_eq!(str, from_tb64)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_display() {
    display();
}

#[test]
fn test_display() {
    display();
}

/// Checks basic construction, printing, and parsing:
/// - Can construct from a tag string and a binary value
/// - Tag and value match the supplied values
/// - String representation can be generated
/// - Generated string can be parsed
/// - Accessors and parsed string match the supplied values
fn check_tb64(tag: &str, value: &[u8]) {
    let mut tb64 = JsTaggedBase64::new(tag, &value).unwrap();
    let str = format!("{}", &tb64);

    // use web_sys;
    // web_sys::console::log_1(&format!("{}", &tb64).into());

    let parsed = JsTaggedBase64::parse(&str).unwrap();

    assert_eq!(&tb64, &parsed);

    // Do we get back the tag we supplied?
    assert_eq!(parsed.tag(), tag);

    // Do we get back the binary value we supplied?
    assert!(is_equal(&parsed.value(), &value));

    // If we change the tag, do we get back the new tag?
    tb64.set_tag("foo");
    assert_eq!(tb64.tag(), "foo");

    // If we change the value, do we get back the new value?
    tb64.set_value(b"bar");
    assert_eq!(tb64.value(), b"bar");
}

fn tagged_base64_parse() {
    // The empty string is not a valid TaggedBase64.
    assert!(TaggedBase64::parse("").is_err());

    // The tag is alphanumeric with hyphen and underscore.
    // The value here is the base64 encoding of foobar, but
    // the encoding doesn't include the required checksum.
    assert!(TaggedBase64::parse("-_~Zm9vYmFy").is_err());

    // An invalid tag should err.
    assert!(TaggedBase64::parse("&_~wA").is_err());

    // A null value is not allowed.
    let b64_null = encode_config("", TB64_CONFIG);
    let tagged = format!("a~{}", &b64_null);
    assert!(TaggedBase64::parse(&tagged).is_err());

    // The tag can be empty, but the value cannot because the value
    // includes the checksum.
    assert!(TaggedBase64::parse("~").is_err());

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
    assert!(TaggedBase64::new("~", b"").is_err());
    assert!(TaggedBase64::new("a~", b"").is_err());
    assert!(TaggedBase64::new("~b", b"").is_err());
    assert!(TaggedBase64::new("c~d", b"").is_err());
    assert!(TaggedBase64::new("e~f~", b"").is_err());
    assert!(TaggedBase64::new("g~h~i", b"").is_err());
    assert!(TaggedBase64::new("Oh, no!", b"").is_err());
    assert!(TaggedBase64::new("Σ", b"").is_err());

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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_tagged_base64_parse() {
    tagged_base64_parse();
}

#[test]
fn test_tagged_base64_parse() {
    tagged_base64_parse();
}

fn tagged_base64_new_tester() {
    let bv = u128::MAX.to_ne_bytes().to_vec();
    let tb = TaggedBase64::new("BIG", &bv);
    assert!(is_equal(&tb.unwrap().value(), &bv));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_tagged_base64_new() {
    tagged_base64_new_tester();
}

#[test]
fn test_tagged_base64_new() {
    tagged_base64_new_tester();
}

fn tag_accessor() {
    let tag = "Tag47";
    let bits = b"Just some bits";
    let tb64 = TaggedBase64::new(&tag, bits).unwrap();
    assert_eq!(tb64.tag(), tag);
    assert_eq!(tb64.value(), bits);

    let jstb64 = JsTaggedBase64::new(&tag, bits).unwrap();
    assert_eq!(jstb64.tag(), tag);
    assert_eq!(jstb64.value(), bits);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_tag_accessor() {
    tag_accessor();
}

#[test]
fn test_tag_accessor() {
    tag_accessor();
}

fn tag_setter() {
    let tag = "Godzilla";
    let bits = b"forest";
    let mut tb64 = TaggedBase64::new("Bambi", bits).unwrap();
    tb64.set_tag(tag);
    assert_eq!(tb64.tag(), tag);
    assert_eq!(tb64.value(), bits);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_tag_setter() {
    tag_setter();
}

#[test]
fn test_tag_setter() {
    tag_setter();
}

fn value_setter() {
    let tag = "Godzilla";
    let bits = b"forest";
    let new_bits = b"trees";
    let mut tb64 = TaggedBase64::new(tag, bits).unwrap();
    tb64.set_value(new_bits);
    assert_eq!(tb64.tag(), tag);
    assert_eq!(tb64.value(), new_bits);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_value_setter() {
    value_setter();
}

#[test]
fn test_value_setter() {
    value_setter();
}

fn empty_value() {
    let t = TaggedBase64::new("TAG", b"").unwrap();
    assert_eq!(t.tag(), "TAG");
    assert_eq!(t.value(), b"");
    assert_eq!(TaggedBase64::parse("TAG~Ew").unwrap(), t);
    assert_eq!(
        TaggedBase64::parse("A~wA").unwrap(),
        TaggedBase64::new("A", b"").unwrap()
    );
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_empty_value() {
    empty_value();
}

#[test]
fn test_empty_value() {
    empty_value();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_js_new_error() {
    match JsTaggedBase64::new("~", b"oops!") {
        Err(e) => assert_eq!(e, to_jsvalue("An invalid character was found in the tag.")),
        _ => assert!(false),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_error_to_string() {
    assert_eq!(
        JsValue::from(Tb64Error::InvalidByte(66, 42)),
        to_jsvalue("An invalid byte (0x2a) was found at offset 66 while decoding the base64-encoded value. The offset and offending byte are provided.")
    );
}

#[test]
fn test_error_fmt() {
    assert_eq!(
        format!("{}", Tb64Error::InvalidByte(66, 42)),
        "An invalid byte (0x2a) was found at offset 66 while decoding the base64-encoded value. The offset and offending byte are provided.".to_string());
}

#[test]
fn basic_errors() {
    let e = TaggedBase64::new("A/A",&[0]).unwrap_err();
    println!("{:?}: {}", e, e);
    assert!(matches!(e,Tb64Error::InvalidTag));

    let e = TaggedBase64::parse("AA").unwrap_err();
    println!("{:?}: {}", e, e);
    assert!(matches!(e,Tb64Error::MissingDelimiter));

    let e = TaggedBase64::parse("AAA~AAA").unwrap_err();
    println!("{:?}: {}", e, e);
    assert!(matches!(e,Tb64Error::InvalidChecksum));

    let e = TaggedBase64::parse("AAA~").unwrap_err();
    println!("{:?}: {}", e, e);
    assert!(matches!(e,Tb64Error::MissingChecksum));

    let e = TaggedBase64::parse("AAA~AAAAA").unwrap_err();
    println!("{:?}: {}", e, e);
    assert!(matches!(e,Tb64Error::InvalidLength));

    let e = TaggedBase64::parse("AAA~AAF").unwrap_err();
    println!("{:?}: {}", e, e);
    assert!(matches!(e,Tb64Error::InvalidLastSymbol(_,_)));
}

fn one_bit_corruption(tag: u16, data: (Vec<u8>,u8), bit_to_flip: u16) {
    let encoded_tag = TaggedBase64::encode_raw(&tag.to_le_bytes());
    assert_eq!(encoded_tag.len(), 3);

    let (mut data, last_data) = data;
    data.push(last_data);

    let encoded = TaggedBase64::new(&encoded_tag,&data).unwrap();
    let mut encoded_bytes = to_string(&encoded).into_bytes();
    let (ix,shift) = ((bit_to_flip>>3) as usize, bit_to_flip&((1<<3)-1));
    let ix = ix%encoded_bytes.len();
    encoded_bytes[ix] ^= 1<<shift;
    if let Ok(corrupted) = str::from_utf8(&encoded_bytes) {
        println!("{}", TaggedBase64::parse(corrupted).unwrap_err());
    }
}

#[quickcheck]
fn one_bit_corruption_quickcheck(tag: u16, data: (Vec<u8>,u8), bit_to_flip: u16) {
    one_bit_corruption(tag,data,bit_to_flip);
}


