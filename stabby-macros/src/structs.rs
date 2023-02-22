use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, DataStruct, Generics, Visibility};

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    DataStruct { fields, .. }: DataStruct,
    st: TokenStream,
) -> proc_macro2::TokenStream {
    let unbound_generics = &generics.params;
    let mut struct_as_tuples = quote!(());
    let struct_code = match fields {
        syn::Fields::Named(fields) => {
            let fields = fields.named;
            for field in &fields {
                let ty = &field.ty;
                struct_as_tuples = quote!(#st::Tuple2<#struct_as_tuples, #ty>)
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
                struct_as_tuples = quote!(#st::Tuple2<#struct_as_tuples, #ty>)
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics (#fields);
                #[automatically_derived]
                unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #struct_as_tuples: #st::IStable {
                    type IllegalValues = <#struct_as_tuples as #st::IStable>::IllegalValues;
                    type UnusedBits =<#struct_as_tuples as #st::IStable>::UnusedBits;
                    type Size = <#struct_as_tuples as #st::IStable>::Size;
                    type Align = <#struct_as_tuples as #st::IStable>::Align;
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                #(#attrs)*
                #vis struct #ident #generics;
            }
        }
    };
    quote! {
        #struct_code
        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #struct_as_tuples: #st::IStable {
            type IllegalValues = <#struct_as_tuples as #st::IStable>::IllegalValues;
            type UnusedBits =<#struct_as_tuples as #st::IStable>::UnusedBits;
            type Size = <#struct_as_tuples as #st::IStable>::Size;
            type Align = <#struct_as_tuples as #st::IStable>::Align;
        }
    }
}
