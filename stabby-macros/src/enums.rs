//
// Copyright (c) 2023 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   Pierre Avital, <pierre.avital@me.com>
//

use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DataEnum, Generics, Ident, Visibility};

use crate::Unself;

#[derive(Clone, Copy, PartialEq, Eq)]
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
impl core::fmt::Debug for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Repr::Stabby => "stabby",
            Repr::C => "C",
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
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FullRepr {
    repr: Option<Repr>,
    is_c: bool,
}
impl syn::parse::Parse for FullRepr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = FullRepr {
            repr: None,
            is_c: false,
        };
        while !input.is_empty() {
            match input.parse()? {
                Repr::C => this.is_c = true,
                repr => match this.repr {
                    None => this.repr = Some(repr),
                    Some(r) if repr == r => {}
                    _ => return Err(input.error("Determinants may only have one representation. You can use `#[repr(C, u8)]` to use a u8 as determinant while ensuring all variants have their data in C layout.")),
                },
            }
            if !input.is_empty() {
                let _: syn::token::Comma = input.parse()?;
            }
        }
        Ok(this)
    }
}

struct Args {
    version: u32,
    module: proc_macro2::TokenStream,
}
impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Args {
            version: 0,
            module: quote!(),
        };
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "version" => {
                    input.parse::<syn::Token!(=)>()?;
                    this.version = input.parse::<syn::LitInt>()?.to_string().parse().unwrap();
                }
                "module" => {
                    input.parse::<syn::Token!(=)>()?;
                    while !input.is_empty() {
                        if input.peek(syn::Token!(,)) {
                            break;
                        }
                        let token: proc_macro2::TokenTree = input.parse()?;
                        this.module.extend(Some(token))
                    }
                }
                _ => return Err(input.error("Unknown stabby attribute {ident}")),
            }
            _ = input.parse::<syn::Token!(,)>();
        }
        Ok(this)
    }
}
pub fn stabby(
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    data: DataEnum,
    stabby_attrs: &proc_macro::TokenStream,
) -> TokenStream {
    let st = crate::tl_mod();
    let Args { version, module } = syn::parse(stabby_attrs.clone()).unwrap();
    let unbound_generics = &generics.params;
    let mut repr: Option<FullRepr> = None;
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
    if matches!(
        repr,
        Some(FullRepr {
            repr: Some(Repr::Stabby),
            is_c: true
        })
    ) {
        panic!("#[repr(C)] and #[repr(stabby)] connot be combined")
    }
    if data.variants.is_empty() {
        todo!("empty enums are not supported by stabby YET")
    }
    let mut layout = quote!(());
    let DataEnum { variants, .. } = &data;
    let mut has_non_empty_fields = false;
    let unit: syn::Type = syn::parse2(quote!(())).unwrap();
    let mut report = crate::Report::r#enum(ident.to_string(), version, module.clone());
    for variant in variants {
        match &variant.fields {
            syn::Fields::Named(f) if matches!(repr, Some(FullRepr { is_c: true, .. })) => {
                has_non_empty_fields = true;
                let mut variant_report =
                    crate::Report::r#struct(variant.ident.to_string(), version, module.clone());
                let mut variant_layout = quote!(());
                for f in &f.named {
                    let ty = f.ty.unself(&ident);
                    variant_layout = quote!(#st::FieldPair<#variant_layout, #ty>);
                    variant_report.add_field(f.ident.as_ref().unwrap().to_string(), ty);
                }
                variant_layout = quote!(#st::Struct<#variant_layout>);
                layout = quote!(#st::Union<#layout, core::mem::ManuallyDrop<#variant_layout>>);
                report.add_field(variant.ident.to_string(), variant_report);
            }
            syn::Fields::Named(_) => {
                panic!("stabby only supports named fields in #[repr(C, u*)] enums");
            }
            syn::Fields::Unnamed(f) => {
                if f.unnamed.len() != 1 && matches!(repr, Some(FullRepr { is_c: true, .. })) {
                    has_non_empty_fields = true;
                    let mut variant_report =
                        crate::Report::r#struct(variant.ident.to_string(), version, module.clone());
                    let mut variant_layout = quote!(());
                    for (n, f) in f.unnamed.iter().enumerate() {
                        let ty = f.ty.unself(&ident);
                        variant_layout = quote!(#st::FieldPair<#variant_layout, #ty>);
                        variant_report.add_field(n.to_string(), ty);
                    }
                    variant_layout = quote!(#st::Struct<#variant_layout>);
                    layout = quote!(#st::Union<#layout, core::mem::ManuallyDrop<#variant_layout>>);
                    report.add_field(variant.ident.to_string(), variant_report);
                } else {
                    assert_eq!(
                        f.unnamed.len(),
                        1,
                        "stabby only supports multiple fields per enum variant in #[repr(C, u*)] enums"
                    );
                    has_non_empty_fields = true;
                    let f = f.unnamed.first().unwrap();
                    let ty = f.ty.unself(&ident);
                    layout = quote!(#st::Union<#layout, core::mem::ManuallyDrop<#ty>>);
                    report.add_field(variant.ident.to_string(), ty);
                }
            }
            syn::Fields::Unit => {
                report.add_field(variant.ident.to_string(), unit.clone());
            }
        }
    }
    let mut deprecation = None;
    let trepr = match repr
        .as_ref()
        .and_then(|r| r.repr.or_else(|| r.is_c.then_some(Repr::C)))
    {
        None | Some(Repr::Stabby) => {
            if !has_non_empty_fields {
                panic!("Your enum doesn't have any field with values: use #[repr(C)] or (preferably) #[repr(u*)] instead")
            }
            return repr_stabby(
                &new_attrs,
                &vis,
                &ident,
                &generics,
                data.clone(),
                report,
                repr.is_none(),
            );
        }
        Some(Repr::C) => {
            let msg = format!("{repr:?} stabby doesn't support variable size repr and implicitly replaces repr(C) with repr(C, u8), you can silence this warning by picking an explict fixed-size repr");
            deprecation = Some(quote!(#[deprecated = #msg]));
            Repr::U8
        } // TODO: Remove support for `#[repr(C)]` alone on the next API-breaking release
        Some(Repr::U8) => Repr::U8,
        Some(Repr::U16) => Repr::U16,
        Some(Repr::U32) => Repr::U32,
        Some(Repr::U64) => Repr::U64,
        Some(Repr::Usize) => Repr::Usize,
        Some(Repr::I8) => Repr::I8,
        Some(Repr::I16) => Repr::I16,
        Some(Repr::I32) => Repr::I32,
        Some(Repr::I64) => Repr::I64,
        Some(Repr::Isize) => Repr::Isize,
    };
    let reprid = quote::format_ident!("{trepr:?}");
    let reprattr = if repr.map_or(false, |r| r.is_c) {
        quote!(#[repr(C, #reprid)])
    } else {
        quote!(#[repr(#reprid)])
    };
    layout = quote!(#st::Tuple<#reprid, #layout>);
    report.tyty = crate::Tyty::Enum(trepr);
    let report_bounds = report.bounds();
    let ctype = cfg!(feature = "experimental-ctypes").then(|| {
        let ctype = report.crepr();
        quote! {type CType = #ctype;}
    });
    let size_bug = format!(
        "{ident}'s size was mis-evaluated by stabby, this is definitely a bug and may cause UB, please file an issue"
    );
    let align_bug = format!(
        "{ident}'s align was mis-evaluated by stabby, this is definitely a bug and may cause UB, please file an issue"
    );
    let reprc_bug = format!(
        "{ident}'s CType was mis-evaluated by stabby, this is definitely a bug and may cause UB, please file an issue"
    );
    let ctype_assert = cfg!(feature = "experimental-ctypes").then(|| {
        quote! {if core::mem::size_of::<Self>() != core::mem::size_of::<<Self as #st::IStable>::CType>() || core::mem::align_of::<Self>() != core::mem::align_of::<<Self as #st::IStable>::CType>() {
            panic!(#reprc_bug)
        }}
    });
    let assertion = generics
        .params
        .is_empty()
        .then(|| quote!(const _: () = {<#ident as #st::IStable>::ID;};));

    quote! {
        #(#new_attrs)*
        #reprattr
        #deprecation
        #vis enum #ident #generics {
            #variants
        }
        #assertion
        #[automatically_derived]
        // SAFETY: This is generated by `stabby`, and checks have been added to detect potential issues.
        unsafe impl #generics #st::IStable for #ident <#unbound_generics> where #report_bounds #layout: #st::IStable {
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
            type ContainsIndirections = <#layout as #st::IStable>::ContainsIndirections;
            #ctype
            const REPORT: &'static #st::report::TypeReport = & #report;
            const ID: u64 ={
                #ctype_assert
                if core::mem::size_of::<Self>() != <<Self as #st::IStable>::Size as #st::Unsigned>::USIZE {
                    panic!(#size_bug)
                }
                if core::mem::align_of::<Self>() != <<Self as #st::IStable>::Align as #st::Unsigned>::USIZE {
                    panic!(#align_bug)
                }
                #st::report::gen_id(Self::REPORT)
            };
        }
    }
}

struct Variant {
    ident: Ident,
    field: Option<syn::Field>,
    attrs: Vec<Attribute>,
}
impl From<&syn::Variant> for Variant {
    fn from(value: &syn::Variant) -> Self {
        let syn::Variant {
            ident,
            fields,
            discriminant: None,
            attrs,
            ..
        } = value.clone()
        else {
            panic!("#[repr(stabby)] enums do not support explicit discriminants")
        };
        let field = match fields {
            syn::Fields::Unit => None,
            syn::Fields::Unnamed(mut f) => {
                let field = f.unnamed.pop().map(|f| f.into_value());
                assert!(f.unnamed.is_empty());
                field
            }
            syn::Fields::Named(_) => unreachable!(),
        };
        Variant {
            ident,
            field,
            attrs,
        }
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
impl<'a> FromIterator<&'a syn::Variant> for Variants {
    fn from_iter<T: IntoIterator<Item = &'a syn::Variant>>(iter: T) -> Self {
        Self {
            variants: Vec::from_iter(iter.into_iter().map(Into::into)),
        }
    }
}
impl Variants {
    fn recursion<'a, U, LeafFn: FnMut(&'a Variant) -> U, JoinFn: FnMut(U, U) -> U>(
        variants: &'a [Variant],
        leaf: &mut LeafFn,
        join: &mut JoinFn,
    ) -> U {
        if variants.len() > 1 {
            let (left, right) = variants.split_at(variants.len() / 2);
            let left = Self::recursion(left, leaf, join);
            let right = Self::recursion(right, leaf, join);
            join(left, right)
        } else {
            leaf(&variants[0])
        }
    }
    fn map<'a, U, LeafFn: FnMut(&'a Variant) -> U, JoinFn: FnMut(U, U) -> U>(
        &'a self,
        mut leaf: LeafFn,
        mut join: JoinFn,
    ) -> U {
        Self::recursion(&self.variants, &mut leaf, &mut join)
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
        mut leaf: LeafFn,
        mut join: JoinFn,
        final_join: FinalJoinFn,
    ) -> V {
        let (left, right) = self.variants.split_at(self.variants.len() / 2);
        let left = Self::recursion(left, &mut leaf, &mut join);
        let right = Self::recursion(right, &mut leaf, &mut join);
        final_join(left, right)
    }
}

pub(crate) fn repr_stabby(
    attrs: &Vec<Attribute>,
    vis: &Visibility,
    ident: &Ident,
    generics: &Generics,
    data: DataEnum,
    report: crate::Report,
    check: bool,
) -> TokenStream {
    let st = crate::tl_mod();
    let unbound_generics = crate::utils::unbound_generics(&generics.params);
    let generics_without_defaults = crate::utils::generics_without_defaults(&generics.params);
    let data_variants = data.variants;
    if data_variants.len() < 2 {
        panic!("#[repr(stabby)] doesn't support single-member enums");
    }
    let variants = data_variants.iter().collect::<Variants>();
    let vty = variants
        .iter()
        .map(|v| v.field.as_ref().map(|f| &f.ty))
        .collect::<Vec<_>>();
    let vtyref = vty
        .iter()
        .map(|v| v.map(|ty| quote!(&'st_lt #ty)))
        .collect::<Vec<_>>();
    let vtymut = vty
        .iter()
        .map(|v| v.map(|ty| quote!(&mut #ty)))
        .collect::<Vec<_>>();
    let vid = variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let vattrs = variants.iter().map(|v| &v.attrs);
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
                quote!(#aty: #st::IDeterminantProvider<#bty>, #bty: #st::IStable, #abound #bbound),
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
            |Variant { ident, field, .. }| match field {
                Some(_) => quote!(#ident),
                None => quote!(|_| #ident()),
            },
            |a, b| quote!(move |this| this.#matcher(#a, #b)),
            |a, b| quote!(self.0.#matcher(#a, #b)),
        )
    };
    let matcher_ctx = |matcher| {
        variants.map_with_finalizer(
            |Variant { ident, field, .. }| match field {
                Some(_) => quote!(#ident),
                None => quote!(|stabby_ctx, _| #ident(stabby_ctx)),
            },
            |a, b| quote!(move |stabby_ctx, this| this.#matcher(stabby_ctx, #a, #b)),
            |a, b| quote!(self.0.#matcher(stabby_ctx, #a, #b)),
        )
    };
    let owned_matcher = matcher(quote!(match_owned));
    let ref_matcher = matcher(quote!(match_ref));
    let mut_matcher = variants.map_with_finalizer(
        |Variant { ident, field, .. }| match field {
            Some(_) => quote!(|mut a| #ident(&mut*a)),
            None => quote!(|_| #ident()),
        },
        |a, b| quote!(move |mut this| this.match_mut(#a, #b)),
        |a, b| quote!(self.0.match_mut(#a, #b)),
    );
    let owned_matcher_ctx = matcher_ctx(quote!(match_owned_ctx));
    let ref_matcher_ctx = matcher_ctx(quote!(match_ref_ctx));
    let mut_matcher_ctx = variants.map_with_finalizer(
        |Variant { ident, field, .. }| match field {
            Some(_) => quote!(|stabby_ctx, mut a| #ident(stabby_ctx, &mut *a)),
            None => quote!(|stabby_ctx, _| #ident(stabby_ctx)),
        },
        |a, b| quote!(move |stabby_ctx, mut this| this.match_mut_ctx(stabby_ctx, #a, #b)),
        |a, b| quote!(self.0.match_mut_ctx(stabby_ctx, #a, #b)),
    );
    let layout = &result;

    let bounds2 = generics.where_clause.as_ref().map(|c| &c.predicates);
    let bounds = quote!(#bounds #bounds2);

    let report_bounds = report.bounds();
    let enum_as_struct = quote! {
        #(#attrs)*
        #vis struct #ident #generics (#result) where #report_bounds #bounds;
    };
    let check = check.then(|| {
        let opt_id = quote::format_ident!("ReprCLayoutFor{ident}");
        let optdoc = format!("Returns true if the layout for [`{ident}`] is smaller than what `#[repr(C)]` would have generated for it.");
        quote! {
            #[allow(dead_code)]
            #[repr(u8)]
            enum #opt_id <#generics_without_defaults> where #bounds {
                #data_variants
            }
            impl<#generics_without_defaults> #ident <#unbound_generics> where #report_bounds #bounds #layout: #st::IStable  {
                #[doc = #optdoc]
                pub const fn has_optimal_layout() -> bool {
                    core::mem::size_of::<Self>() < core::mem::size_of::<#opt_id<#unbound_generics>>()
                }
            }
        }
    });
    let ctype = cfg!(feature = "experimental-ctypes").then(|| {
        quote! {type CType = <#layout as #st::IStable>::CType;}
    });
    let assertions= generics.params.is_empty().then(||{
        let check = check.is_some().then(||{
            let sub_optimal_message = format!(
                "{ident}'s layout is sub-optimal, reorder fields or use `#[repr(stabby)]` to silence this error."
            );
            quote!(
                if !<#ident>::has_optimal_layout() {
                    panic!(#sub_optimal_message)
                })
        });
        let size_bug = format!(
            "{ident}'s size was mis-evaluated by stabby, this is definitely a bug and may cause UB, please fill an issue"
        );
        let align_bug = format!(
            "{ident}'s align was mis-evaluated by stabby, this is definitely a bug and may cause UB, please fill an issue"
        );
        quote! {
            const _: () = {
                #check
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
        #enum_as_struct
        #check
        #assertions
        #[automatically_derived]
        // SAFETY: This is generated by `stabby`, and checks have been added to detect potential issues.
        unsafe impl <#generics_without_defaults> #st::IStable for #ident < #unbound_generics > where #report_bounds #bounds #layout: #st::IStable {
            type ForbiddenValues = <#layout as #st::IStable>::ForbiddenValues;
            type UnusedBits =<#layout as #st::IStable>::UnusedBits;
            type Size = <#layout as #st::IStable>::Size;
            type Align = <#layout as #st::IStable>::Align;
            type HasExactlyOneNiche = #st::B0;
            type ContainsIndirections = <#layout as #st::IStable>::ContainsIndirections;
            #ctype
            const REPORT: &'static #st::report::TypeReport = & #report;
            const ID: u64 = #st::report::gen_id(Self::REPORT);
        }
        #[automatically_derived]
        impl #generics #ident < #unbound_generics > where #report_bounds #bounds {
            #(
                #[allow(non_snake_case)]
                #(#vattrs)*
                pub fn #vid(#cparams) -> Self {
                    Self (#constructors)
                }
            )*
            #[allow(non_snake_case)]
            /// Equivalent to `match self`.
            pub fn match_owned<StabbyOut, #(#fnvid: FnOnce(#vty) -> StabbyOut,)*>(self, #(#vid: #fnvid,)*) -> StabbyOut {
                #owned_matcher
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &self`.
            pub fn match_ref<'st_lt, StabbyOut, #(#fnvid: FnOnce(#vtyref) -> StabbyOut,)*>(&'st_lt self, #(#vid: #fnvid,)*) -> StabbyOut {
                #ref_matcher
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &mut self`.
            pub fn match_mut<'st_lt, StabbyOut, #(#fnvid: FnOnce(#vtymut) -> StabbyOut,)*>(&'st_lt mut self, #(#vid: #fnvid,)*) -> StabbyOut {
                #mut_matcher
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match self`, but allows you to pass common arguments to all closures to make the borrow checker happy.
            pub fn match_owned_ctx<StabbyOut, StabbyCtx, #(#fnvid: FnOnce(StabbyCtx, #vty) -> StabbyOut,)*>(self, stabby_ctx: StabbyCtx, #(#vid: #fnvid,)*) -> StabbyOut {
                #owned_matcher_ctx
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &self`, but allows you to pass common arguments to all closures to make the borrow checker happy.
            pub fn match_ref_ctx<'st_lt, StabbyCtx, StabbyOut, #(#fnvid: FnOnce(StabbyCtx, #vtyref) -> StabbyOut,)*>(&'st_lt self, stabby_ctx: StabbyCtx, #(#vid: #fnvid,)*) -> StabbyOut {
                #ref_matcher_ctx
            }
            #[allow(non_snake_case)]
            /// Equivalent to `match &mut self`, but allows you to pass common arguments to all closures to make the borrow checker happy.
            pub fn match_mut_ctx<'st_lt, StabbyCtx, StabbyOut, #(#fnvid: FnOnce(StabbyCtx, #vtymut) -> StabbyOut,)*>(&'st_lt mut self, stabby_ctx: StabbyCtx, #(#vid: #fnvid,)*) -> StabbyOut {
                #mut_matcher_ctx
            }
        }
    }
}
