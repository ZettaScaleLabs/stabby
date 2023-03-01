use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DataStruct, Generics, Visibility};

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    DataStruct { fields, .. }: DataStruct,
) -> proc_macro2::TokenStream {
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let generics_without_defaults = crate::utils::generics_without_defaults(&generics.params);
    let mut layout = None;
    let struct_code = match fields {
        syn::Fields::Named(fields) => {
            let fields = fields.named;
            for field in &fields {
                let ty = &field.ty;
                layout = Some(
                    layout.map_or_else(|| quote!(#ty), |layout| quote!(#st::Tuple2<#layout, #ty>)),
                )
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics {
                    #fields
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields.unnamed;
            for field in &fields {
                let ty = &field.ty;
                layout = Some(
                    layout.map_or_else(|| quote!(#ty), |layout| quote!(#st::Tuple2<#layout, #ty>)),
                )
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics (#fields);
            }
        }
        syn::Fields::Unit => {
            quote! {
                #(#attrs)*
                #vis struct #ident #generics;
            }
        }
    };
    let layout = layout.unwrap_or_else(|| quote!(()));
    quote! {
        #struct_code
        #[automatically_derived]
        unsafe impl <#generics_without_defaults> #st::IStable for #ident <#unbound_generics> where #layout: #st::IStable {
            type IllegalValues = <#layout as #st::IStable>::IllegalValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = <#layout as #st::IStable>::HasExactlyOneNiche;
        }
    }
}
