use proc_macro2::TokenStream;
use quote::quote;
use syn::{PatType, Receiver, Signature, TraitItemMethod, TraitItemType};

pub fn stabby(
    syn::ItemTrait {
        attrs,
        vis,
        unsafety,
        auto_token,
        trait_token,
        ident,
        generics,
        colon_token,
        supertraits,
        brace_token: _,
        items,
    }: syn::ItemTrait,
    st: TokenStream,
) -> TokenStream {
    let mut vtable_fields: Vec<TokenStream> = Vec::new();
    // let mut assoc_types = Vec::new();
    for item in &items {
        match item {
            syn::TraitItem::Method(method) => {
                let TraitItemMethod {
                    sig:
                        Signature {
                            asyncness,
                            unsafety,
                            ident,
                            generics,
                            inputs,
                            variadic,
                            output,
                            ..
                        },
                    ..
                } = method;
                if asyncness.is_some() {
                    panic!("stabby doesn't support async functions");
                }
                if variadic.is_some() {
                    panic!("stabby doesn't support variadics");
                }
                if generics
                    .params
                    .iter()
                    .any(|g| !matches!(g, syn::GenericParam::Lifetime(_)))
                {
                    panic!("generic methods are not trait object safe")
                }
                let inputs = inputs.iter().map(|input| match input {
                    syn::FnArg::Receiver(Receiver {
                        reference: Some(_),
                        mutability,
                        ..
                    }) => {
                        quote!(&#mutability Self)
                    }
                    syn::FnArg::Typed(PatType { ty, .. }) => {
                        quote!(#ty)
                    }
                    _ => panic!("fn (self, ...) is not trait safe"),
                });
                vtable_fields.push(quote!(#ident: extern "C" #unsafety fn (#(#inputs),*) #output,))
            }
            syn::TraitItem::Type(ty) => {
                let TraitItemType {
                    // attrs,
                    // ident,
                    // generics,
                    // colon_token,
                    // bounds,
                    // default,
                    ..
                } = ty;
                todo!("stabby doesn't support associated types YET")
            }
            syn::TraitItem::Const(_) => panic!("associated consts are not trait object safe"),
            syn::TraitItem::Macro(_) => panic!("sabby can't see through macros in traits"),
            syn::TraitItem::Verbatim(tt) => {
                panic!("stabby failed to parse this token stream {}", tt)
            }
            _ => panic!("unexpected element in trait"),
        }
    }
    let vtident = quote::format_ident!("Vt{ident}");
    let hasvtident = quote::format_ident!("HasVt{ident}");
    let unbound_generics = &generics.params;
    quote! {
        pub struct #vtident <  #unbound_generics > {
            drop: extern "C" fn (&mut Self),
            #(#vtable_fields)*
        }
        pub trait #hasvtident #generics : #ident < #unbound_generics > {
            const VTABLE: #vtident <  #unbound_generics >;
            fn vtable(&self) -> &'static #vtident <  #unbound_generics > {&Self::VTABLE}
        }
        #(#attrs)*
        #vis #unsafety #auto_token #trait_token #ident #generics #colon_token #supertraits {
            #(#items)*
        }
    }
}
