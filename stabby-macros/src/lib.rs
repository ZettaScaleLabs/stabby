use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, ExprArray, ExprLit, Lit};

#[proc_macro]
pub fn holes(input: TokenStream) -> TokenStream {
    let ExprArray { elems: holes, .. } = syn::parse::<ExprArray>(input).unwrap();
    assert_eq!(holes.len(), 4);
    let mut bits = Vec::with_capacity(256);
    for spec in holes {
        let Expr::Lit(ExprLit{lit: Lit::Int(spec), ..}) = spec else {panic!()};
        let spec: u64 = spec.base10_parse().unwrap();
        for i in 0..64 {
            bits.push(if (spec >> i) & 1 != 0 {
                quote!(::typenum::B1)
            } else {
                quote!(::typenum::B0)
            });
        }
    }
    quote!(stabby_traits::holes::Holes<#(#bits,)*>).into()
}
mod tyops;
#[proc_macro]
pub fn tyeval(tokens: TokenStream) -> TokenStream {
    tyops::tyeval(&tokens.into()).into()
}

#[proc_macro_attribute]
pub fn stabby(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let in_stabby = !attrs.is_empty();
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = syn::parse(tokens).unwrap();
    match data {
        syn::Data::Struct(data) => structs::stabby(attrs, vis, ident, generics, data, in_stabby),
        syn::Data::Enum(_) => panic!("stabby doesn't support enums YET"),
        syn::Data::Union(_) => panic!("stabby doesn't support unions YET"),
    }
    .into()
}
mod structs;
