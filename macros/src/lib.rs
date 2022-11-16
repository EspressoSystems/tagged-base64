// Copyright (c) 2022 Espresso Systems (espressosys.com)
#![no_std]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta};

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
/// #[macro_use] extern crate tagged_base64_macros;
/// use ark_serialize::*;
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
/// # #[tagged("PRIM")]
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
/// # #[tagged("PRIM")]
/// # #[derive(Clone, CanonicalSerialize, CanonicalDeserialize, /* any other derives */)]
/// # struct CryptoPrim(ark_bls12_381::Fr);
/// # let crypto_prim = CryptoPrim(ark_bls12_381::Fr::rand(&mut ChaChaRng::from_seed([42; 32])));
/// serde_json::to_string(&crypto_prim).unwrap();
/// ```
/// which will produce a tagged base64 string like
/// "PRIM~8oaujwbov8h4eEq7HFpqW6mIXhVbtJGxLUgiKrGpMCoJ".
#[proc_macro_attribute]
pub fn tagged(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as Item);
    let (name, generics) = match &input {
        Item::Struct(item) => (&item.ident, &item.generics),
        Item::Enum(item) => (&item.ident, &item.generics),
        _ => panic!("expected struct or enum"),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let tag: &dyn quote::ToTokens = match args.as_slice() {
        [NestedMeta::Lit(tag)] => tag,
        [NestedMeta::Meta(Meta::Path(path))] => path,
        x => panic!(
            "`tagged` takes one argument, the tag, as a string literal or expression, found {:?}",
            x
        ),
    };
    let output = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "tagged_base64::TaggedBase64", into = "tagged_base64::TaggedBase64")]
        // Override the inferred bound for Serialize/Deserialize impls. If we're converting to and
        // from CanonicalBytes as an intermediate, the impls should work for any generic parameters.
        #[serde(bound = "")]
        #input

        impl #impl_generics tagged_base64::Tagged for #name #ty_generics #where_clause {
            fn tag() -> ark_std::string::String {
                ark_std::string::String::from(#tag)
            }
        }

        impl #impl_generics core::convert::TryFrom<tagged_base64::TaggedBase64>
            for #name #ty_generics
        #where_clause
        {
            type Error = tagged_base64::Tb64Error;
            fn try_from(t: tagged_base64::TaggedBase64) -> Result<Self, Self::Error> {
                if t.tag() == <#name #ty_generics>::tag() {
                    <Self as CanonicalDeserialize>::deserialize(t.as_ref())
                        .map_err(|_| tagged_base64::Tb64Error::InvalidData)
                } else {
                    Err(tagged_base64::Tb64Error::InvalidTag)
                }
            }
        }

        impl #impl_generics core::convert::From<#name #ty_generics> for tagged_base64::TaggedBase64
            #where_clause
        {
            fn from(x: #name #ty_generics) -> Self {
                (&x).into()
            }
        }

        impl #impl_generics core::convert::From<&#name #ty_generics> for tagged_base64::TaggedBase64
            #where_clause
        {
            fn from(x: &#name #ty_generics) -> Self {
                let mut bytes = ark_std::vec![];
                x.serialize(&mut bytes).unwrap();
                Self::new(&<#name #ty_generics>::tag(), &bytes).unwrap()
            }
        }

        impl #impl_generics ark_std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ark_std::fmt::Formatter<'_>) -> ark_std::fmt::Result {
                ark_std::write!(
                    f, "{}",
                    tagged_base64::TaggedBase64::from(self)
                )
            }
        }

        impl #impl_generics ark_std::str::FromStr for #name #ty_generics #where_clause {
            type Err = tagged_base64::Tb64Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use core::convert::TryFrom;
                Self::try_from(tagged_base64::TaggedBase64::from_str(s)?)
                    .map_err(|_| tagged_base64::Tb64Error::InvalidData)
            }
        }
    };
    output.into()
}
