use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataEnum, Generics, Ident, Visibility};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repr {
    Stabby,
    C,
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
}
impl syn::parse::Parse for Repr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id: syn::Ident = input.parse()?;
        match id.to_string().as_str() {
            "C" => Ok(Self::C),
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "usize" => Ok(Self::Usize),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "isize" => Ok(Self::Isize),
            "stabby" => Ok(Self::Stabby),
            _ => Err(input.error("Unexpected repr, only `u*` and `stabby` are supported")),
        }
    }
}

pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: DataEnum,
) -> TokenStream {
    let st = crate::tl_mod();
    let unbound_generics = &generics.params;
    let mut repr = None;
    let repr_ident = quote::format_ident!("repr");
    let mut new_attrs = Vec::with_capacity(attrs.len());
    for a in attrs {
        if a.path.is_ident(&repr_ident) {
            if repr.is_none() {
                repr = Some(a.parse_args().unwrap())
            } else {
                panic!("multiple reprs are forbidden")
            }
        } else {
            new_attrs.push(a)
        }
    }
    if data.variants.is_empty() {
        todo!("empty enums are not supported by stabby YET")
    }
    let mut layout = quote!(());
    let DataEnum { variants, .. } = data;
    let mut has_non_empty_fields = false;
    for variant in &variants {
        match &variant.fields {
            syn::Fields::Named(_) => {
                panic!("stabby does not support named fields in enum variants")
            }
            syn::Fields::Unnamed(f) => {
                assert_eq!(
                    f.unnamed.len(),
                    1,
                    "stabby only supports one field per enum variant"
                );
                has_non_empty_fields = true;
                let f = f.unnamed.first().unwrap();
                let ty = &f.ty;
                layout = quote!(#st::Union<#layout, core::mem::ManuallyDrop<#ty>>)
            }
            syn::Fields::Unit => {}
        }
    }
    let repr = repr.unwrap_or(Repr::Stabby);
    let declaration = {
        // 'stabby: {
        let repr = match repr {
            Repr::Stabby => {
                if !has_non_empty_fields {
                    panic!("Your enum doesn't have any field with values: use #[repr(C)] or #[repr(u*)] instead")
                }
                todo!("#[repr(stabby)] isn't supported YET");
                // break 'stabby quote!();
            }
            Repr::C => "u8",
            Repr::U8 => "u8",
            Repr::U16 => "u16",
            Repr::U32 => "u32",
            Repr::U64 => "u64",
            Repr::Usize => "usize",
            Repr::I8 => "i8",
            Repr::I16 => "i16",
            Repr::I32 => "i32",
            Repr::I64 => "i64",
            Repr::Isize => "isize",
        };
        let repr = quote::format_ident!("{}", repr);
        layout = quote!(#st::Tuple2<#repr, #layout>);
        quote! {
            #(#new_attrs)*
            #[repr(#repr)]
            #vis enum #ident #generics {
                #variants
            }
        }
    };
    quote! {
        #declaration

        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #layout: #st::IStable {
            type IllegalValues = <#layout as #st::IStable>::IllegalValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
        }
    }
}
