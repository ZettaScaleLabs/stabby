use proc_macro::TokenStream;
use quote::{quote, ToTokens};
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

fn assert_stable(st: &impl ToTokens, ty: impl ToTokens) -> proc_macro2::TokenStream {
    quote!(let _ = #st::AssertStable::<#ty>(::core::marker::PhantomData);)
}

#[proc_macro_attribute]
pub fn stabby(attrs: TokenStream, tokens: TokenStream) -> TokenStream {
    let in_stabby = !attrs.is_empty();
    let st = if in_stabby {
        quote!(::stabby_traits::type_layouts)
    } else {
        quote!(::stabby::stabby_traits::type_layouts)
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
            syn::Data::Enum(_) => panic!("stabby doesn't support enums YET"),
            syn::Data::Union(_) => panic!("stabby doesn't support unions YET"),
        }
    } else if let Ok(syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    }) = syn::parse(tokens)
    {
        let syn::Signature {
            abi,
            inputs,
            output,
            ..
        } = &sig;
        assert!(
            abi.is_none(),
            "stabby will attribute a stable ABI to your function on its own"
        );
        let mut stable_asserts = Vec::new();
        if let syn::ReturnType::Type(_, ty) = output {
            stable_asserts.push(assert_stable(&st, ty));
        }
        stable_asserts.extend(inputs.iter().map(|i| match i {
            syn::FnArg::Receiver(_) => assert_stable(&st, quote!(Self)),
            syn::FnArg::Typed(syn::PatType { ty, .. }) => assert_stable(&st, ty),
        }));
        quote! {
            #(#attrs)*
            #vis extern "C" #sig {
                #(#stable_asserts)*
                #block
            }
        }
    } else {
        panic!("Expected a type declaration, a trait declaration or a function declaration")
    }
    .into()
}
mod structs;
