use std::ops::Deref;

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
    let DataEnum { variants, .. } = &data;
    let mut has_non_empty_fields = false;
    for variant in variants {
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
        let repr = match repr {
            Repr::Stabby => {
                if !has_non_empty_fields {
                    panic!("Your enum doesn't have any field with values: use #[repr(C)] or #[repr(u*)] instead")
                }
                return repr_stabby(&new_attrs, &vis, &ident, &generics, data);
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
        layout = quote!(#st::FieldPair<#repr, #layout>);
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
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
        }
    }
}

struct Variant {
    ident: Ident,
    field: Option<syn::Field>,
}
impl From<syn::Variant> for Variant {
    fn from(value: syn::Variant) -> Self {
        let syn::Variant {
            ident,
            fields,
            discriminant: None,
            ..
        } = value else {panic!("#[repr(stabby)] enums do not support explicit discriminants")};
        let field = match fields {
            syn::Fields::Unit => None,
            syn::Fields::Unnamed(mut f) => {
                let field = f.unnamed.pop().map(|f| f.into_value());
                assert!(f.unnamed.is_empty());
                field
            }
            syn::Fields::Named(_) => unreachable!(),
        };
        Variant { ident, field }
    }
}
struct Variants {
    variants: Vec<Variant>,
}
impl Deref for Variants {
    type Target = Vec<Variant>;
    fn deref(&self) -> &Self::Target {
        &self.variants
    }
}
impl FromIterator<syn::Variant> for Variants {
    fn from_iter<T: IntoIterator<Item = syn::Variant>>(iter: T) -> Self {
        Self {
            variants: Vec::from_iter(iter.into_iter().map(Into::into)),
        }
    }
}
impl Variants {
    fn map<'a, U, LeafFn: FnMut(&'a Variant) -> U, JoinFn: FnMut(U, U) -> U>(
        &'a self,
        leaf: LeafFn,
        mut join: JoinFn,
    ) -> U {
        let mut buffer = self.variants.iter().map(leaf).collect::<Vec<_>>();
        while buffer.len() > 1 {
            let len = buffer.len();
            let mut iter = buffer.into_iter();
            buffer = Vec::with_capacity(len / 2 + 1);
            while let Some(a) = iter.next() {
                if let Some(b) = iter.next() {
                    buffer.push(join(a, b))
                } else {
                    buffer.push(a)
                }
            }
        }
        buffer.pop().unwrap()
    }
    fn map_with_finalizer<
        'a,
        U,
        V,
        LeafFn: FnMut(&'a Variant) -> U,
        JoinFn: FnMut(U, U) -> U,
        FinalJoinFn: FnOnce(U, U) -> V,
    >(
        &'a self,
        leaf: LeafFn,
        mut join: JoinFn,
        final_join: FinalJoinFn,
    ) -> V {
        let mut buffer = self.variants.iter().map(leaf).collect::<Vec<_>>();
        while buffer.len() > 2 {
            let len = buffer.len();
            let mut iter = buffer.into_iter();
            buffer = Vec::with_capacity(len / 2 + 1);
            while let Some(a) = iter.next() {
                if let Some(b) = iter.next() {
                    buffer.push(join(a, b))
                } else {
                    buffer.push(a)
                }
            }
        }
        let last = buffer.pop().unwrap();
        final_join(buffer.pop().unwrap(), last)
    }
}

pub fn repr_stabby(
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    data: DataEnum,
) -> TokenStream {
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let variants = data.variants;
    if variants.len() < 2 {
        panic!("#[repr(stabby)] doesn't support single-member enums");
    }
    let variants = variants.into_iter().collect::<Variants>();
    let vty = variants
        .iter()
        .map(|v| v.field.as_ref().map(|f| &f.ty))
        .collect::<Vec<_>>();
    let vtyref = vty.iter().map(|v| v.map(|ty| quote!(&'st_lt #ty)));
    let vtymut = vty.iter().map(|v| v.map(|ty| quote!(&'st_lt mut #ty)));
    let vid = variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let fnvid = vid
        .iter()
        .map(|i| quote::format_ident!("{i}Fn"))
        .collect::<Vec<_>>();
    let (result, bounds) = variants.map(
        |Variant { field, .. }| match field.as_ref() {
            Some(syn::Field { ty, .. }) => (quote!(#ty), quote!()),
            None => (quote!(()), quote!()),
        },
        |(aty, abound), (bty, bbound)| {
            (
                quote!(#st::Result<#aty, #bty>),
                quote!((#aty, #bty): #st::IDiscriminantProvider, #abound #bbound),
            )
        },
    );
    let mut cparams = Vec::new();
    let constructors = variants.map(
        |v| {
            let ovid = match &v.field {
                Some(syn::Field { ty, .. }) => {
                    cparams.push(quote!(value: #ty));
                    quote!(value)
                }
                None => {
                    cparams.push(quote!());
                    quote!(())
                }
            };
            vec![ovid]
        },
        |a, b| {
            let mut r = Vec::with_capacity(a.len() + b.len());
            for v in a {
                r.push(quote!(#st::Result::Ok(#v)))
            }
            for v in b {
                r.push(quote!(#st::Result::Err(#v)))
            }
            r
        },
    );
    let matcher = |matcher| {
        variants.map_with_finalizer(
            |Variant { ident, field }| match field {
                Some(_) => quote!(#ident),
                None => quote!(|_| #ident()),
            },
            |a, b| quote!(move |this| this.#matcher(#a, #b)),
            |a, b| quote!(self.0.#matcher(#a, #b)),
        )
    };
    let owned_matcher = matcher(quote!(match_owned));
    let ref_matcher = matcher(quote!(match_ref));
    let mut_matcher = matcher(quote!(match_mut));
    let layout = &result;
    quote! {
        #(#attrs)*
        #vis struct #ident #generics (#result) where #bounds;
        #[automatically_derived]
        unsafe impl #generics #st::IStable for #ident < #unbound_generics > where #bounds #layout: #st::IStable {
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
        }
        #[automatically_derived]
        impl #generics #ident < #unbound_generics > where #bounds {
            #(
                #[allow(non_snake_case)]
                pub fn #vid(#cparams) -> Self {
                    Self (#constructors)
                }
            )*
            #[allow(non_snake_case)]
            pub fn match_owned<U, #(#fnvid: FnOnce(#vty) -> U,)*>(self, #(#vid: #fnvid,)*) -> U {
                #owned_matcher
            }
            #[allow(non_snake_case)]
            pub fn match_ref<'st_lt, U, #(#fnvid: FnOnce(#vtyref) -> U,)*>(&'st_lt self, #(#vid: #fnvid,)*) -> U {
                #ref_matcher
            }
            #[allow(non_snake_case)]
            pub fn match_mut<'st_lt, U, #(#fnvid: FnOnce(#vtymut) -> U,)*>(&'st_lt mut self, #(#vid: #fnvid,)*) -> U {
                #mut_matcher
            }
        }
    }
}
