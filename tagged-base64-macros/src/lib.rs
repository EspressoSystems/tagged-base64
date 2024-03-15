// Copyright (c) 2022 Espresso Systems (espressosys.com)
#![no_std]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta};

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
    let mut compressed = false;
    let mut checked = false;
    let (tag, marks): (&dyn quote::ToTokens, _) = match args.as_slice() {
        [NestedMeta::Lit(tag), marks @ ..] => (tag, marks),
        [NestedMeta::Meta(Meta::Path(path)), marks @ ..] => (path, marks),
        x => panic!(
            "`tagged` takes at least one argument, the tag, as a string literal or expression, found {:?}",
            x
        ),
    };
    marks.iter().for_each(|attr| match attr {
        NestedMeta::Meta(Meta::Path(path)) => {
            if path.is_ident("compressed") {
                compressed = true;
            } else if path.is_ident("checked") {
                checked = true;
            } else {
                panic!("Unkown tagged argument, should be either \"compressed\" or \"checked\".")
            }
        }
        _ => panic!("Unkown tagged argument, should be either \"compressed\" or \"checked\"."),
    });
    let serialize_token = if compressed {
        quote!(serialize_compressed)
    } else {
        quote!(serialize_uncompressed)
    };
    let deserialize_token = if compressed {
        if checked {
            quote!(deserialize_compressed)
        } else {
            quote!(deserialize_compressed_unchecked)
        }
    } else if checked {
        quote!(deserialize_uncompressed)
    } else {
        quote!(deserialize_uncompressed_unchecked)
    };

    #[cfg(feature = "serde")]
    let struct_def = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "tagged_base64::TaggedBase64", into = "tagged_base64::TaggedBase64")]
        // Override the inferred bound for Serialize/Deserialize impls. If we're converting to and
        // from CanonicalBytes as an intermediate, the impls should work for any generic parameters.
        #[serde(bound = "")]
        #input
    };
    #[cfg(not(feature = "serde"))]
    let struct_def = &input;

    let output = quote! {
        #struct_def

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
                use core::convert::TryInto;
                (&t).try_into()
            }
        }

        impl #impl_generics core::convert::TryFrom<&tagged_base64::TaggedBase64>
            for #name #ty_generics
        #where_clause
        {
            type Error = tagged_base64::Tb64Error;
            fn try_from(t: &tagged_base64::TaggedBase64) -> Result<Self, Self::Error> {
                if t.tag() == <#name #ty_generics as tagged_base64::Tagged>::tag() {
                    <Self as CanonicalDeserialize>::#deserialize_token(t.as_ref())
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
                CanonicalSerialize::#serialize_token(x, &mut bytes).unwrap();
                Self::new(&<#name #ty_generics as tagged_base64::Tagged>::tag(), &bytes).unwrap()
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
