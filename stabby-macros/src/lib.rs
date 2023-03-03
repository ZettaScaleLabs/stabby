use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse::Parser, DeriveInput, Expr, ExprArray, ExprLit, Lit, TypeParamBound};

pub(crate) fn tl_mod() -> proc_macro2::TokenStream {
    match proc_macro_crate::crate_name("stabby")
        .expect("Couldn't find `stabby` in your dependencies")
    {
        proc_macro_crate::FoundCrate::Itself => quote!(crate::abi),
        proc_macro_crate::FoundCrate::Name(crate_name) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            quote!(#crate_name::abi)
        }
    }
}

#[proc_macro_attribute]
pub fn stabby(_attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    if let Ok(DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    }) = syn::parse(tokens.clone())
    {
        match data {
            syn::Data::Struct(data) => structs::stabby(attrs, vis, ident, generics, data),
            syn::Data::Enum(data) => enums::stabby(attrs, vis, ident, generics, data),
            syn::Data::Union(data) => unions::stabby(attrs, vis, ident, generics, data),
        }
    } else if let Ok(fn_spec) = syn::parse(tokens.clone()) {
        functions::stabby(fn_spec)
    } else if let Ok(trait_spec) = syn::parse(tokens) {
        traits::stabby(trait_spec)
    } else {
        panic!("Expected a type declaration, a trait declaration or a function declaration")
    }
    .into()
}

#[proc_macro]
pub fn vtable(tokens: TokenStream) -> TokenStream {
    let st = tl_mod();
    let bounds =
        syn::punctuated::Punctuated::<TypeParamBound, syn::token::Add>::parse_separated_nonempty
            .parse(tokens)
            .unwrap();
    let mut vt = quote!(#st::vtable::VtDrop);
    for bound in bounds {
        match &bound {
            TypeParamBound::Trait(t) => vt = quote!(< dyn #t as #st::vtable::CompoundVt >::Vt<#vt>),
            TypeParamBound::Lifetime(_) => todo!(),
        }
    }
    vt.into()
}

mod enums;
mod functions;
mod structs;
mod traits;
mod unions;
pub(crate) mod utils;

#[proc_macro]
pub fn holes(input: TokenStream) -> TokenStream {
    let st = tl_mod();
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
    quote!(#st::holes::Holes<#(#bits,)*>).into()
}

mod tyops;
#[proc_macro]
pub fn tyeval(tokens: TokenStream) -> TokenStream {
    tyops::tyeval(&tokens.into()).into()
}
