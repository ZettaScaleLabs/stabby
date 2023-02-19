use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DataStruct, Generics, Visibility};

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    DataStruct { fields, .. }: DataStruct,
    in_stabby: bool,
) -> proc_macro2::TokenStream {
    let st = if in_stabby {
        quote!(::stabby_traits)
    } else {
        quote!(::stabby::stabby_traits)
    };
    match fields {
        syn::Fields::Named(fields) => {
            let fields = fields.named;
            let unbound_generics = &generics.params;
            let mut bounds = quote!();
            for field in &fields {
                let ty = &field.ty;
                bounds = quote! {
                    #bounds
                    #ty: #st::Stable,
                };
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics {
                    #fields
                }
                #[automatically_derived]
                unsafe impl #generics #st::Stable for #ident <#unbound_generics> where #bounds {
                    type Niches = #st::End<#st::U0>;
                    type Align = #st::U0;
                    type Size = #st::U0;
                }
            }
        }
        syn::Fields::Unnamed(_) => {
            panic!("stabby doesn't support tuple-like structs (nor does it intend to atm)")
        }
        syn::Fields::Unit => {
            let unbound_generics = &generics.params;
            quote! {
                #(#attrs)*
                #vis struct #ident #generics;
                #[automatically_derived]
                unsafe impl #generics #st::Stable for #ident <#unbound_generics> {
                    type Niches = #st::End<#st::U0>;
                    type Align = #st::U0;
                    type Size = #st::U0;
                }
            }
        }
    }
}
