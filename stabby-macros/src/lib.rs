use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    ConstParam, DeriveInput, Expr, ExprArray, ExprLit, GenericParam, LifetimeDef, Lit, TypeParam,
};

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
    quote!(stabby::type_layouts::holes::Holes<#(#bits,)*>).into()
}

mod tyops;
#[proc_macro]
pub fn tyeval(tokens: TokenStream) -> TokenStream {
    tyops::tyeval(&tokens.into()).into()
}

#[proc_macro_attribute]
pub fn stabby(_attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let st = match proc_macro_crate::crate_name("stabby")
        .expect("Couldn't find `stabby` in your dependencies")
    {
        proc_macro_crate::FoundCrate::Itself => quote!(crate::type_layouts),
        proc_macro_crate::FoundCrate::Name(crate_name) => {
            let crate_name = Ident::new(&crate_name, Span::call_site());
            quote!(#crate_name::type_layouts)
        }
    };
    if let Ok(DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    }) = syn::parse(tokens.clone())
    {
        match data {
            syn::Data::Struct(data) => structs::stabby(attrs, vis, ident, generics, data, st),
            syn::Data::Enum(data) => enums::stabby(attrs, vis, ident, generics, data, st),
            syn::Data::Union(data) => unions::stabby(attrs, vis, ident, generics, data, st),
        }
    } else if let Ok(fn_spec) = syn::parse(tokens.clone()) {
        functions::stabby(fn_spec, st)
    } else if let Ok(trait_spec) = syn::parse(tokens) {
        traits::stabby(trait_spec, st)
    } else {
        panic!("Expected a type declaration, a trait declaration or a function declaration")
    }
    .into()
}
#[derive(Clone, Default)]
pub(crate) struct SeparatedGenerics {
    pub lifetimes: Vec<proc_macro2::TokenStream>,
    pub types: Vec<proc_macro2::TokenStream>,
    pub consts: Vec<proc_macro2::TokenStream>,
}
impl quote::ToTokens for SeparatedGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for l in &self.lifetimes {
            tokens.extend(quote!(#l,));
        }
        for l in &self.types {
            tokens.extend(quote!(#l,));
        }
        for l in &self.consts {
            tokens.extend(quote!(#l,));
        }
    }
}
pub(crate) fn unbound_generics<'a>(
    generics: impl IntoIterator<Item = &'a GenericParam>,
) -> SeparatedGenerics {
    let mut this = SeparatedGenerics::default();
    for g in generics {
        match g {
            GenericParam::Type(TypeParam { ident, .. }) => this.types.push(quote!(#ident)),
            GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => {
                this.lifetimes.push(quote!(#lifetime))
            }
            GenericParam::Const(ConstParam { ident, .. }) => this.consts.push(quote!(#ident)),
        }
    }
    this
}
pub(crate) fn generics_without_defaults<'a>(
    generics: impl IntoIterator<Item = &'a GenericParam>,
) -> SeparatedGenerics {
    let mut this = SeparatedGenerics::default();
    for g in generics {
        match g {
            GenericParam::Type(TypeParam { ident, bounds, .. }) => {
                this.types.push(quote!(#ident: #bounds))
            }
            GenericParam::Lifetime(LifetimeDef {
                lifetime, bounds, ..
            }) => this.lifetimes.push(quote!(#lifetime: #bounds)),
            GenericParam::Const(ConstParam { ident, .. }) => this.consts.push(quote!(#ident)),
        }
    }
    this
}

mod enums;
mod functions;
mod structs;
mod traits;
mod unions;
