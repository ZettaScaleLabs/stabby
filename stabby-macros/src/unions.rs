use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataUnion, Generics, Ident, Visibility};
pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: DataUnion,
    st: TokenStream,
) -> TokenStream {
    let DataUnion {
        union_token: _,
        fields,
    } = &data;
    let unbound_generics = &generics.params;
    let mut layout = quote!(());
    for field in &fields.named {
        let ty = &field.ty;
        layout = quote!(#st::Union<#layout, #ty>)
    }
    quote! {
        #(#attrs)*
        #[repr(C)]
        #vis union #ident #generics
            #fields

        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #layout: #st::IStable {
            type IllegalValues = #st::End;
            type UnusedBits = #st::End;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
        }
    }
}
