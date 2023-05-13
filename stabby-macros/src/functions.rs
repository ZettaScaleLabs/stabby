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

use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

include!(concat!(env!("OUT_DIR"), "/env_vars.rs"));

pub struct Attrs {
    unsend: bool,
    unsync: bool,
    lt: syn::Lifetime,
}
impl Default for Attrs {
    fn default() -> Self {
        Attrs {
            unsend: false,
            unsync: false,
            lt: syn::Lifetime::new("'static", proc_macro2::Span::call_site()),
        }
    }
}
enum Attr {
    Unsend,
    Unsync,
    Lt(syn::Lifetime),
}
impl syn::parse::Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Lifetime) {
            Ok(Attr::Lt(input.parse()?))
        } else {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "unsend" => Ok(Attr::Unsend),
                "unsync" => Ok(Attr::Unsync),
                _ => Err(syn::Error::new(ident.span(), "Unsupported attribute for `stabby` on functions: only lifetimes, `unsend` and `unsync` are supported"))
            }
        }
    }
}
impl syn::parse::Parse for Attrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Self {
            unsend: false,
            unsync: false,
            lt: syn::Lifetime {
                apostrophe: input.span(),
                ident: quote::format_ident!("static"),
            },
        };
        for attr in syn::punctuated::Punctuated::<Attr, syn::Token!(,)>::parse_terminated(input)? {
            match attr {
                Attr::Unsend => this.unsend = true,
                Attr::Unsync => this.unsync = true,
                Attr::Lt(lt) => this.lt = lt,
            }
        }
        Ok(this)
    }
}

pub fn stabby(attrs: Attrs, fn_spec: syn::ItemFn) -> proc_macro2::TokenStream {
    let st = crate::tl_mod();
    fn assert_stable(st: &impl ToTokens, ty: impl ToTokens) -> proc_macro2::TokenStream {
        quote!(let _ = #st::AssertStable::<#ty>(::core::marker::PhantomData);)
    }
    let Attrs { unsend, unsync, lt } = attrs;

    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = fn_spec;
    let syn::Signature {
        abi,
        inputs,
        output,
        asyncness,
        generics,
        unsafety,
        constness,
        ident,
        ..
    } = &sig;
    let abi = match abi {
        None | Some(syn::Abi { name: None, .. }) => {
            quote!(extern "C")
        }
        Some(syn::Abi {
            name: Some(name), ..
        }) if [
            "C", "system", "stdcall", "aapcs", "cdecl", "fastcall", "win64", "sysv64",
        ]
        .contains(&name.value().as_str()) =>
        {
            quote!(#abi)
        }
        _ => panic!("stabby functions must use a stable calling convention"),
    };
    let mut stable_asserts = Vec::new();
    if let syn::ReturnType::Type(_, ty) = output {
        stable_asserts.push(assert_stable(&st, ty));
    }
    stable_asserts.extend(inputs.iter().map(|i| match i {
        syn::FnArg::Receiver(_) => assert_stable(&st, quote!(Self)),
        syn::FnArg::Typed(syn::PatType { ty, .. }) => assert_stable(&st, ty),
    }));
    let (output, block) = if asyncness.is_some() {
        let mut future = match output {
            syn::ReturnType::Default => quote!(#st::future::Future<Output=()>),
            syn::ReturnType::Type(_, ty) => quote!(#st::future::Future<Output=#ty>),
        };
        if !unsend {
            future = quote!(#future + Send)
        }
        if !unsync {
            future = quote!(#future + Sync)
        }
        let vt: TokenStream = crate::vtable(future.into()).into();
        let output = quote!( -> #st::Dyn<#lt, Box<()>, #vt>);
        (output, quote!(Box::new(async {#block}).into()))
    } else {
        (quote!(#output), quote!(#block))
    };
    quote! {
        #(#attrs)*
        #vis #unsafety #constness #abi fn #ident #generics (#inputs) #output {
            #(#stable_asserts)*
            #block
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CanarySpec(u16);
impl CanarySpec {
    pub const NONE: Self = Self(0);
    pub const RUSTC: Self = Self(1);
    pub const OPT_LEVEL: Self = Self(1 << 1);
    pub const DEBUG: Self = Self(1 << 2);
    pub const NUM_JOBS: Self = Self(1 << 3);
    pub const TARGET: Self = Self(1 << 4);
    pub const HOST: Self = Self(1 << 5);
    pub const PARANOID: Self = Self(0b111111);
    pub const ARRAY: &[(&'static str, Self)] = &[
        ("paranoid", Self::PARANOID),
        ("none", Self::NONE),
        ("rustc", Self::RUSTC),
        ("opt_level", Self::OPT_LEVEL),
        ("target", Self::TARGET),
        ("num_jobs", Self::NUM_JOBS),
        ("debug", Self::DEBUG),
        ("host", Self::HOST),
    ];
}
impl FromStr for CanarySpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::PARANOID);
        }
        let known = Self::ARRAY.iter().map(|s| s.0).collect::<Vec<_>>();
        if s.chars()
            .any(|c| !(c.is_alphabetic() || c == ' ' || c == ','))
        {
            return Err(format!("Invalid canary-spec: `{s}` must be a comma-separated list of canary identifiers ({known:?})"));
        }
        let mut this = Self::NONE;
        for request in s.split(',').map(|r| r.trim()) {
            let Some(spec) = Self::ARRAY.iter().find_map(|(name, spec)| (*name == request).then_some(*spec)) else {return Err(format!("Unknown canary `{request}` (known canaries: {known:?})"))};
            this = this | spec;
        }
        Ok(this)
    }
}
impl IntoIterator for CanarySpec {
    type Item = CanarySpec;
    type IntoIter = Canaries;
    fn into_iter(self) -> Self::IntoIter {
        Canaries {
            spec: self.0,
            shift: 0,
        }
    }
}
pub struct Canaries {
    spec: u16,
    shift: u16,
}
impl Iterator for Canaries {
    type Item = CanarySpec;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let candidate = 1 << self.shift;
            if candidate > self.spec {
                return None;
            }
            self.shift += 1;
            if candidate & self.spec != 0 {
                return Some(CanarySpec(candidate));
            }
        }
    }
}
impl core::ops::BitOr for CanarySpec {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl core::ops::BitAnd for CanarySpec {
    type Output = bool;
    fn bitand(self, rhs: Self) -> Self::Output {
        (self.0 & rhs.0) != 0
    }
}
fn fix(source: &str) -> String {
    use core::fmt::Write;
    let mut result = String::with_capacity(source.len());
    for c in source.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            result.push(c)
        } else {
            write!(result, "_{:x}_", c as u32).unwrap()
        }
    }
    result
}
impl core::fmt::Display for CanarySpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let this = *self;
        write!(f, "_canary")?;
        if this & Self::RUSTC {
            write!(
                f,
                "_rustc_{RUSTC_MAJOR}_{RUSTC_MINOR}_{RUSTC_PATCH}_{}",
                &RUSTC_COMMIT[..8]
            )?
        }
        if this & Self::OPT_LEVEL {
            write!(f, "_optlevel{OPT_LEVEL}")?
        }
        if this & Self::DEBUG {
            write!(f, "_debug{}", DEBUG.parse::<bool>().unwrap() as u8)?
        }
        if this & Self::NUM_JOBS {
            write!(f, "_numjobs{NUM_JOBS}")?
        }
        if this & Self::TARGET {
            write!(f, "_target_{}", fix(TARGET))?
        }
        if this & Self::HOST {
            write!(f, "_host_{}", fix(HOST))?
        }
        Ok(())
    }
}

fn export_canaried(fn_spec: syn::ItemFn) -> proc_macro2::TokenStream {
    let canaries = (0..=5)
        .map(|i| quote::format_ident!("{}{}", fn_spec.sig.ident, CanarySpec(1 << i).to_string()));
    quote! {
        #[no_mangle]
        #[allow(improper_ctypes_definitions)]
        #fn_spec
        #(
            #[no_mangle]
            pub extern "C" fn #canaries() {}
        )*
    }
}

fn export_with_report(fn_spec: syn::ItemFn) -> proc_macro2::TokenStream {
    let syn::Signature {
        asyncness,
        unsafety,
        abi,
        ident,
        inputs,
        output,
        ..
    } = fn_spec.sig.clone();
    let st = crate::tl_mod();
    let stabbied = quote::format_ident!("{ident}_stabbied");
    let report = quote::format_ident!("{stabbied}_report");
    let def = stabby(Attrs::default(), fn_spec);
    let signature = quote!(#asyncness #unsafety #abi fn(#inputs) #output);
    let stabbied = stabby(
        Attrs::default(),
        syn::parse2(quote::quote! {
            extern "C" fn #stabbied(report: &#st::report::TypeReport) -> Option<#signature> {
                <#signature as #st::IStable>::REPORT.is_compatible(report).then_some(#ident)
            }
        })
        .unwrap(),
    );
    let report = stabby(
        Attrs::default(),
        syn::parse2(quote! {
            extern "C" fn #report() -> &'static #st::report::TypeReport {
                <#signature as #st::IStable>::REPORT
            }
        })
        .unwrap(),
    );
    quote::quote!(
        #[no_mangle]
        #def
        #[no_mangle]
        #stabbied
        #[no_mangle]
        #report
    )
}

struct ExportArgs {
    canaried: bool,
}
impl syn::parse::Parse for ExportArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = ExportArgs { canaried: false };
        if input.is_empty() {
            return Ok(args);
        }
        let id: syn::Ident = input.parse()?;
        if id == "canaries" {
            args.canaried = true
        } else {
            panic!("Unsupported argument to `stabby::export. `canaried` is the only currently supported arg.")
        }
        Ok(args)
    }
}

pub fn export(
    macro_attrs: proc_macro::TokenStream,
    fn_spec: syn::ItemFn,
) -> proc_macro2::TokenStream {
    let args = syn::parse::<ExportArgs>(macro_attrs).unwrap();
    if args.canaried {
        export_canaried(fn_spec)
    } else {
        export_with_report(fn_spec)
    }
}

struct IdentEqStr {
    ident: syn::Ident,
    str: syn::LitStr,
}
impl syn::parse::Parse for IdentEqStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let _: syn::Token!(=) = input.parse()?;
        Ok(IdentEqStr {
            ident,
            str: input.parse()?,
        })
    }
}

struct ImportArgs {
    canaries: Option<CanarySpec>,
    link_args: proc_macro2::TokenStream,
}
impl syn::parse::Parse for ImportArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = ImportArgs {
            canaries: None,
            link_args: quote!(),
        };
        for IdentEqStr { ident, str } in
            input.parse_terminated::<_, syn::Token!(,)>(IdentEqStr::parse)?
        {
            if ident == "canaries" {
                args.canaries = Some(CanarySpec::from_str(&str.value()).unwrap())
            } else {
                args.link_args.extend(quote!(#ident = #str,))
            }
        }
        Ok(args)
    }
}

pub fn import(
    macro_attrs: proc_macro::TokenStream,
    fn_decl: syn::ItemForeignMod,
) -> proc_macro2::TokenStream {
    let ImportArgs {
        canaries,
        link_args,
    } = syn::parse(macro_attrs).unwrap();
    let syn::ItemForeignMod {
        attrs, abi, items, ..
    } = &fn_decl;
    let st = crate::tl_mod();
    let modid = quote::format_ident!("_stabbymod_{}", rand::random::<u128>());
    let mut externs = Vec::new();
    let mut interns = Vec::new();
    let mut intern_ids = Vec::new();
    match canaries {
        Some(canaries) => {
            for item in items {
                match item {
                    syn::ForeignItem::Fn(syn::ForeignItemFn { sig:  syn::Signature { ident,  generics, inputs, output,   .. }, vis, ..}) => {
                        externs.push(quote!(#item));
                        let mut canary_ids = Vec::new();
                        for canary in canaries {
                            let id = quote::format_ident!("{ident}{}", canary.to_string());
                            externs.push(quote!(pub(crate) fn #id();));
                            canary_ids.push(id);
                        }
                        let canaries_fn = quote::format_ident!("{ident}_canaries_fn");
                        let canaried = quote::format_ident!("{ident}_canaried");
                        let sig = quote!(unsafe #abi fn #generics(#inputs)#output);
                        interns.push(quote!{
                            extern "C" fn #canaries_fn() {
                                unsafe{#(#canary_ids();)*}
                            }

                            #[allow(non_upper_case_globals)]
                            pub static #canaried: #st::checked_import::CanariedImport<#sig> = #st::checked_import::CanariedImport::new(#ident as #sig, #canaries_fn as extern "C" fn());
                        });
                        intern_ids.push(quote!(#vis use #modid::#canaried as #ident;))
                    },
                    syn::ForeignItem::Static(_) => todo!("stabby doesn't support importing statics yet. Move this to a separate extern block, and use `#[link]`"),
                    syn::ForeignItem::Type(_) => {externs.push(quote!(#item))},
                    _ => todo!("Unsupported item in a stabby import: {}", quote!(#item)),
                }
            }
        }
        None => {
            for item in items {
                match item {
                    syn::ForeignItem::Fn(syn::ForeignItemFn { sig: syn::Signature { ident,  inputs, output, asyncness, unsafety, generics, .. }, vis, ..}) => {
                        assert!(asyncness.is_none(), "the async keyword is not supported in non-canaried extern blocks");
                        let stabbied = quote::format_ident!("{ident}_stabbied");
                        let report = quote::format_ident!("{stabbied}_report");
                        let signature = quote!(#unsafety #abi fn #generics(#inputs)#output);
                        externs.push(quote!{
                            fn #report() -> &'static #st::report::TypeReport;
                            fn #stabbied(report: & #st::report::TypeReport) -> Option<#signature>;
                        });
                        interns.push(quote!{
                            #[allow(non_upper_case_globals)]
                            pub static #ident: #st::checked_import::CheckedImport<#signature> = #st::checked_import::CheckedImport::new(#stabbied, #report, <#signature as #st::IStable>::REPORT);
                        });
                        intern_ids.push(quote!(#vis use #modid::#ident;));
                    },
                    syn::ForeignItem::Static(_) => todo!("stabby doesn't support importing statics yet. Move this to a separate extern block, and use `#[link]`"),
                    syn::ForeignItem::Type(_) => {externs.push(quote!(#item))},
                    _ => todo!("Unsupported item in a stabby import: {}", quote!(#item)),
                }
            }
        }
    }
    quote! {
        mod #modid {
            #(#attrs)*
            #[link(#link_args)]
            #abi {
                #(#externs)*
            }
            #(#interns)*
        }
        #(#intern_ids)*
    }
}
