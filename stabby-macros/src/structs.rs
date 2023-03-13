use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, DataStruct, Generics, Visibility};

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    DataStruct {
        fields, semi_token, ..
    }: DataStruct,
    stabby_attrs: &proc_macro::TokenStream,
) -> proc_macro2::TokenStream {
    let mut opt = match stabby_attrs.to_string().as_str() {
        "no_opt" => false,
        "" => true,
        _ => panic!("Unkown stabby attributes {stabby_attrs}"),
    };
    opt &= generics.params.is_empty();
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let generics_without_defaults = crate::utils::generics_without_defaults(&generics.params);
    let where_clause = &generics.where_clause;
    let clauses = where_clause.as_ref().map(|w| &w.predicates);
    let mut layout = None;
    let struct_code = match &fields {
        syn::Fields::Named(fields) => {
            let fields = &fields.named;
            for field in fields {
                let ty = &field.ty;
                layout = Some(
                    layout.map_or_else(|| quote!(#ty), |layout| quote!(#st::Tuple2<#layout, #ty>)),
                )
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics #where_clause {
                    #fields
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            for field in fields {
                let ty = &field.ty;
                layout = Some(
                    layout.map_or_else(|| quote!(#ty), |layout| quote!(#st::Tuple2<#layout, #ty>)),
                )
            }
            quote! {
                #(#attrs)*
                #[repr(C)]
                #vis struct #ident #generics #where_clause (#fields);
            }
        }
        syn::Fields::Unit => {
            quote! {
                #(#attrs)*
                #vis struct #ident #generics #where_clause;
            }
        }
    };
    let layout = layout.unwrap_or_else(|| quote!(()));
    let opt_id = quote::format_ident!("OptimizedLayoutFor{ident}");
    let assertion = opt.then(|| {
        let sub_optimal_message = format!(
            "{ident}'s layout is sub-optimal, reorder fields or use `#[stabby::stabby(no_opt)]`"
        );
        let size_bug = format!(
            "{ident}'s size was mis-evaluated by stabby, this is a definitely a bug and may cause UB, please fill an issue"
        );
        let align_bug = format!(
            "{ident}'s align was mis-evaluated by stabby, this is a definitely a bug and may cause UB, please fill an issue"
        );
        quote! {
            const _: () = {
                if !<#ident>::has_optimal_layout() {
                    panic!(#sub_optimal_message)
                }
                if core::mem::size_of::<#ident>() != <<#ident as #st::IStable>::Size as #st::Unsigned>::USIZE {
                    panic!(#size_bug)
                }
                if core::mem::align_of::<#ident>() != <<#ident as #st::IStable>::Align as #st::Unsigned>::USIZE {
                    panic!(#align_bug)
                }
            };
        }
    });
    quote! {
        #struct_code

        #[automatically_derived]
        unsafe impl <#generics_without_defaults> #st::IStable for #ident <#unbound_generics> where #layout: #st::IStable, #clauses {
            type IllegalValues = <#layout as #st::IStable>::IllegalValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = <#layout as #st::IStable>::HasExactlyOneNiche;
        }
        #[allow(dead_code)]
        struct #opt_id #generics #where_clause #fields #semi_token
        #assertion
        impl < #generics_without_defaults > #ident <#unbound_generics> #where_clause {
            pub const fn has_optimal_layout() -> bool {
                core::mem::size_of::<Self>() <= core::mem::size_of::<#opt_id<#unbound_generics>>()
            }
        }
    }
}
