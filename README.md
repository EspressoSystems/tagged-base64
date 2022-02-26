# Tagged Base64

*User-oriented format for binary data.* **Tagged Base64** is intended to be
used in user interfaces including
- URLs
- Text to be copied and pasted

Tagged Base64 does not require additional encoding, such as quoting or
escape sequences. Truncation and other forms of corruption can be
detected with an integrated checksum.

To reduce confusion, the values are prefixed with a tag
intended to disambiguate usage. Although not necessary for
correctness, developers and users may find it convenient to have a
usage hint enabling them to see at a glance whether something is a
transaction ID or a ledger address, etc.

For example,

    TX~QmVhdXRpZnVsEA
    LA~SG9tZbo

Like the base64 value, the tag is also restricted to the URL-safe
base64 character set.

**Note:** The tag may be omitted, but the base64 value cannot because it contains the checksum.

# Standalone Executable

The crate includes a standalone executable for converting to and from Tagged Base64. See `tagged_base64 --help` for usage.

## Rationale

Large binary values don't fit nicely into JavaScript numbers due to
range and representation. JavaScript numbers are represented as 64-bit
floating point numbers. This means that the largest unsigned integer
that can be represented is 2^53 - 1. Moreover, it is very easy to
accidentally coerce a string that looks like a number into a
JavaScript number, thus running the risk of loss of precision, which
is corruption.  Therefore, values are encoded in base64 to allow safe
transit to- and from JavaScript, including in URLs, as well as display
and input in a user interface.

# Prerequisites

In addition to the typical Rust development tools, `wasm-pack` is needed. The Makefile includes a `setup` target to install `wasm-pack`.

The Wasm tests require [Firefox](https://www.mozilla.org/en-US/firefox/new/) and [Chrome](https://www.google.com/chrome/) to be installed.
