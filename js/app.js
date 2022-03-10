// Copyright (c) 2022 Espresso Systems (espressosys.com)

// Node.js Javascript for constructing Tagged Base64

const { crc8 } = require('easy-crc');

function stringToBytes(s) {
    return new TextEncoder().encode(s);
}

function toBytes(s) {
    if(typeof s === 'object' && s instanceof Uint8Array) {
        return s;
    } else if(typeof s === 'string') {
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

if (toTaggedBase64("TX", "") != "TX~1w") {
    console.log('toTaggedBase64("TX", "") is wrong. Should return "TX~1w".');
}

if (toTaggedBase64("TR", "hi") != "TR~aGkR") {
    console.log('toTaggedBase64("TX", "hi") is wrong. Should return "TR~aGkR".');
}

if (toTaggedBase64("TR", toBytes("hi")) != "TR~aGkR") {
    console.log('toTaggedBase64("TX", toBytes("hi")) is wrong. Should return "TR~aGkR".');
}

if (toTaggedBase64("TARNATION", "WAT?! Wat?") != "TARNATION~V0FUPyEgV2F0Pzo") {
    console.log('toTaggedBase64("TARNATION", "WAT?! Wat?") is wrong. Should return "TARNATION~V0FUPyEgV2F0Pzo".');
}

