// Copyright (c) 2022 Espresso Systems (espressosys.com)

// Node.js Javascript for constructing and parsing Tagged Base64
//
// To run this, first
//    npm install easy-crc
// Then
//    node app.js
//


const { crc8 } = require('easy-crc');

function stringToBytes(s) {
    return new TextEncoder().encode(s);
}

function stringFromBytes(b) {
    return new TextDecoder().decode(b);
}

function toBytes(s) {
    if (typeof s === 'object' && s instanceof Uint8Array) {
        return s;
    } else if (typeof s === 'string') {
        return stringToBytes(s);
    }
    return undefined;
}

function bytesConcat(a, b) {
    return new Uint8Array([...a, ...b]);
}

/// Combine tag and data with a checksum to make a TaggedBase64-encoded string.
function toTaggedBase64(tag, data) {
    var data = toBytes(data);
    let cs = crc8('CRC-8', bytesConcat(toBytes(tag), data)) ^ (data.length % 256);
    let csb = Buffer.concat([data, Buffer.from([cs])]);
    let b64 = csb.toString('base64');
    return tag + '~' + b64.replace(/=/g, "")
        .replace(/\+/g, "-")
        .replace(/\//g, "_");
}

/// If tb64 can be parsed as a TaggedBase64 value and the checksum is valid,
/// return the tag and the bytes. Otherwise, return undefined.
function fromTaggedBase64(tb64) {
    const [tag, dataCs] = tb64.split("~");
    if (typeof dataCs == 'undefined') {
        return undefined;
    }
    const bytes = toBytes(Buffer.from(dataCs, 'base64').toString('ascii'));
    const n = bytes.length;
    const cs = bytes[n - 1];
    const data = bytes.subarray(0, n - 1);
    const cs2 = crc8('CRC-8', bytesConcat(toBytes(tag), data)) ^ (data.length % 256);
    if (cs == cs2) {
        return [tag, data];
    } else {
        return undefined;
    }
}

/// Given a TaggedBase64 string with a value encoding a UTF-8 string,
/// parse and extract the tag and the UTF-8 string. Otherwise,
/// return undefined.
function fromTaggedBase64Utf8(tb64) {
    let result = fromTaggedBase64(tb64);
    if (typeof result == 'undefined') {
        return result;
    } else {
        return [result[0], stringFromBytes(result[1])];
    }
}

if (toTaggedBase64("TX", "") !== "TX~1w") {
    console.log('toTaggedBase64("TX", "") is wrong. Should return "TX~1w".');
}

if (toTaggedBase64("TR", "hi") !== "TR~aGkR") {
    console.log('toTaggedBase64("TX", "hi") is wrong. Should return "TR~aGkR".');
}

if (toTaggedBase64("TR", toBytes("hi")) !== "TR~aGkR") {
    console.log('toTaggedBase64("TX", toBytes("hi")) is wrong. Should return "TR~aGkR".');
}

if (toTaggedBase64("TARNATION", "WAT?! Wat?") != "TARNATION~V0FUPyEgV2F0Pzo") {
    console.log('toTaggedBase64("TARNATION", "WAT?! Wat?") is wrong. Should return "TARNATION~V0FUPyEgV2F0Pzo".');
}

const [tag, value] = fromTaggedBase64Utf8('TARNATION~V0FUPyEgV2F0Pzo');
if (tag != 'TARNATION' || value != 'WAT?! Wat?') {
    console.log("fromTaggedBase64Utf8('TARNATION~V0FUPyEgV2F0Pzo') is wrong. Should return [ 'TARNATION', 'WAT?! Wat?' ]");
}

if (typeof fromTaggedBase64Utf8("") != 'undefined') {
    console.log('fromTaggedBase64Utf8("") is wrong. Should be undefined');
}

if (typeof fromTaggedBase64Utf8("a~b") != 'undefined') {
    console.log('fromTaggedBase64Utf8("a~b") is wrong. Should be 0');
}

console.log(toTaggedBase64("YO", new Uint8Array([-1, -1, -1])));
