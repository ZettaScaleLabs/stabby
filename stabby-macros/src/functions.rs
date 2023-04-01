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
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

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
        _ => panic!("stabby traits must use a stable ABI"),
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
    pub const DEFAULT: Self = Self(0b11111);
    pub const PARANOID: Self = Self(0b111111);
    pub const RUSTC: Self = Self(1);
    pub const OPT_LEVEL: Self = Self(1 << 1);
    pub const DEBUG: Self = Self(1 << 2);
    pub const NUM_JOBS: Self = Self(1 << 3);
    pub const TARGET: Self = Self(1 << 4);
    pub const HOST: Self = Self(1 << 5);
    pub const ARRAY: &[(&'static str, Self)] = &[
        ("default", Self::DEFAULT),
        ("paranoid", Self::PARANOID),
        ("none", Self::NONE),
        ("rustc", Self::RUSTC),
        ("opt_level", Self::OPT_LEVEL),
        ("debug", Self::DEBUG),
        ("num_jobs", Self::NUM_JOBS),
        ("target", Self::TARGET),
        ("host", Self::HOST),
    ];
}
impl syn::parse::Parse for CanarySpec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::DEFAULT);
        }
        input.parse::<syn::Token!(=)>()?;
        let span = input.span();
        let request = input.parse::<syn::Ident>()?.to_string();
        CanarySpec::ARRAY
            .iter()
            .find_map(|(name, spec)| (*name == request).then_some(*spec))
            .map_or_else(
                || {
                    let known = Self::ARRAY.iter().map(|s| s.0).collect::<Vec<_>>();
                    Err(syn::Error::new(
                        span,
                        format!("Unknown canary `{request}, try one of {known:?}`"),
                    ))
                },
                Ok,
            )
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
    let def = stabby(Attrs::default(), fn_spec);
    let signature = quote!(#asyncness #unsafety #abi fn(#inputs) #output);
    let report = quote::format_ident!("{stabbied}_report");
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
        if id == "canaried" {
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
